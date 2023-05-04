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
    Ok(())
}

fn runtime<'a, C: Context<'a>>(context: &mut C) -> NeonResult<&'static Runtime> {
    RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| context.throw_error(err.to_string())))
}

fn connect(mut context: FunctionContext) -> JsResult<JsPromise> {
    let rt = runtime(&mut context).unwrap();
    let Ok(connection_link_js) = context.argument::<JsString>(0) else {
        return context.throw_error("No connection link found.")
    };
    let channel = context.channel();
    let (deferred, promise) = context.promise();
    let connection_link = connection_link_js.value(&mut context);

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
    let channel = context.channel();
    let (deferred, promise) = context.promise();
    let Ok(sql_js) = context.argument::<JsString>(0) else {
        return context.throw_error("No SQL found.");
    };
    let sql = sql_js.value(&mut context);
    let rt = RUNTIME.get().unwrap();
    rt.spawn(async move {
        let query = query(&sql).fetch_all(CONNECTION.get().unwrap()).await;
        deferred.settle_with(&channel, move |mut context| match query {
            Ok(results) => {
                let array = context.empty_array();
                results.into_iter().for_each(|row| {
                    let column_len = row.columns().len();
                    let object = context.empty_object();
                    (0..column_len).for_each(|index| {
                        let column = context.string(row.column(index).name());
                        let field = if let Ok(i32_value) = row.try_get::<i32, usize>(index) {
                            context.number(i32_value).as_value(&mut context)
                        } else if let Ok(f64_value) = row.try_get::<f64, usize>(index) {
                            context.number(f64_value).as_value(&mut context)
                        } else if let Ok(string_value) = row.try_get::<&str, usize>(index) {
                            context.string(string_value).as_value(&mut context)
                        } else {
                            context.null().as_value(&mut context)
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
