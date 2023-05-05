use neon::prelude::*;
use once_cell::sync::OnceCell;
use sqlx::{query, Column, Row, SqlitePool};
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();
static CONNECTION: OnceCell<SqlitePool> = OnceCell::new();

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("connect", connect).unwrap();
    cx.export_function("execute", execute).unwrap();
    if let Err(err) =
        RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
    {
        Err(err)
    } else {
        Ok(())
    }
}

fn connect(mut context: FunctionContext) -> JsResult<JsPromise> {
    if CONNECTION.get().is_some() {
        return context.throw_error("Connection to database has already been initalized, could not initalize connection twice.");
    };
    if RUNTIME.get().is_none() {
        return context.throw_error(
            "Could not get the Tokio runtime, which is required for connecting to database.",
        );
    };
    let Ok(connection_link_js) = context.argument::<JsString>(0) else {
        return context.throw_error("No connection link found.")
    };
    let channel = context.channel();
    let (deferred, promise) = context.promise();
    let connection_link = connection_link_js.value(&mut context);
    let rt = RUNTIME.get().unwrap();
    rt.spawn(async move {
        let connection_result = SqlitePool::connect(&connection_link).await;
        deferred.settle_with(&channel, move |mut context| {
            if let Ok(connection) = connection_result {
                CONNECTION.set(connection).unwrap();
                Ok(context.boolean(true))
            } else {
                context.throw_error("Unable to connect with provided info.")
            }
        });
    });

    Ok(promise)
}

fn execute(mut context: FunctionContext) -> JsResult<JsPromise> {
    if CONNECTION.get().is_none() {
        return context.throw_error("Connection to database has not been initalized, please call connect() method before execute.");
    };
    let Some(rt) = RUNTIME.get() else {
        return context.throw_error(
            "Could not get the Tokio runtime, which is required for executing SQL.",
        );
    };
    let channel = context.channel();
    let (deferred, promise) = context.promise();
    let Ok(sql_js) = context.argument::<JsString>(0) else {
        return context.throw_error("No SQL found.");
    };
    let args_js_option = context.argument_opt(1);
    let sql = sql_js.value(&mut context);
    let args = if let Some(args_js) = args_js_option {
        let Ok(args_js_array) = args_js.downcast::<JsArray, _>(&mut context) else {
             return context.throw_error("Unable to parse arguments.")
        };
        let vec = args_js_array
            .to_vec(&mut context)
            .unwrap()
            .into_iter()
            .map(|js_value| {
                if js_value.downcast::<JsNull, _>(&mut context).is_ok() {
                    None
                } else {
                    let js_string = js_value.to_string(&mut context).unwrap();
                    Some(js_string.value(&mut context))
                }
            })
            .collect::<Vec<Option<String>>>();
        Some(vec)
    } else {
        None
    };
    rt.spawn(async move {
        let mut query = query(&sql);
        if let Some(args_vec) = args {
            for arg in args_vec {
                query = query.bind(arg);
            }
        }
        let execute = query.fetch_all(CONNECTION.get().unwrap()).await;
        deferred.settle_with(&channel, move |mut context| match execute {
            Ok(results) => {
                let array = context.empty_array();
                results.into_iter().for_each(|row| {
                    let column_len = row.columns().len();
                    let object = context.empty_object();
                    (0..column_len).for_each(|index| {
                        let column = context.string(row.column(index).name());
                        let field = if let Ok(option_i32_value) =
                            row.try_get::<Option<i32>, usize>(index)
                        {
                            if let Some(i32_value) = option_i32_value {
                                context.number(i32_value).as_value(&mut context)
                            } else {
                                context.null().as_value(&mut context)
                            }
                        } else if let Ok(f64_value) = row.try_get::<f64, usize>(index) {
                            context.number(f64_value).as_value(&mut context)
                        } else if let Ok(string_value) = row.try_get::<&str, usize>(index) {
                            context.string(string_value).as_value(&mut context)
                        } else {
                            context.undefined().as_value(&mut context)
                        };
                        object.set(&mut context, column, field).unwrap();
                    });
                    let array_len = array.len(&mut context);
                    array.set(&mut context, array_len, object).unwrap();
                });
                Ok(array)
            }
            Err(error) => context.throw_error(error.to_string()),
        });
    });

    Ok(promise)
}
