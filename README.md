# rusty-sqlite3

A SQLite3 Client built with Rust Library [SQLx](https://github.com/launchbadge/sqlx) and [Neon](https://neon-bindings.com/).

## Installing rusty-sqlite3

Installing rusty-sqlite3 requires [Rust Programming Language](https://rustup.rs/) installed in your device.

Then you can install the project with npm. In the project directory, run:

```sh
$ npm install rusty-sqlite3
```

This fully installs rust-sqlite3, including installing any dependencies and running the build.

## APIs

> Database operations in SQLx are asynchronous, so all API provided by this package will return a `Promise`.

### `connect(url)`

Create a new connection with a database file, or open an in memory SQLite3 database. Return `true` if connected successfully.

> Both reconnect and disconnect are currently impossible.

#### Example

1. Open a database file.

    ```javascript
    > await rustSqlite3.connect("sqlite://database.sqlite")
    true
    ```

2. Open an in memory SQLite3 database.

    ```javascript
    > await rustSqlite3.connect("sqlite://database.sqlite")
    true
    ```

### `execute(sql, [args])`

Execute the provided SQL. Returns the query result with an array of object.

> Arguments are optional. For more information of binding a value with query, see [SQLx API Documentation](https://docs.rs/sqlx/latest/sqlx/query/struct.Query.html#method.bind) for details.

#### Example

    ```javascript
    > await rustSqlite3.execute("select sqlite_version()")
    [ { 'sqlite_version()': '3.38.2' } ]
    ```
