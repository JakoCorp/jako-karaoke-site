//! Grants all capabilities to an existing user by username.
//!
//! ```sh
//! DATABASE_URL=... cargo run --bin set-admin -- <username>
//! ```

use std::env;

use server::capabilities::ALL_CAPABILITIES;

#[tokio::main]
async fn main() {
    let username = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: set-admin <username>");
        std::process::exit(1);
    });

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("error: DATABASE_URL is not set");
        std::process::exit(1);
    });

    let pool = db::connect(&database_url).await.unwrap_or_else(|error| {
        eprintln!("error: failed to connect to database: {error}");
        std::process::exit(1);
    });

    let user = db::queries::users::get_by_username(&pool, &username)
        .await
        .unwrap_or_else(|error| {
            eprintln!("error: database query failed: {error}");
            std::process::exit(1);
        });

    let Some(user) = user else {
        eprintln!("error: no user found with username '{username}'");
        std::process::exit(1);
    };

    let mut conn = pool.acquire().await.unwrap_or_else(|error| {
        eprintln!("error: failed to acquire connection: {error}");
        std::process::exit(1);
    });

    for capability in ALL_CAPABILITIES {
        db::queries::capabilities::add_to_user(&mut conn, user.id, capability)
            .await
            .unwrap_or_else(|error| {
                eprintln!("error: failed to grant '{capability}': {error}");
                std::process::exit(1);
            });
        println!("granted: {capability}");
    }

    println!(
        "done: all capabilities granted to '{username}' ({})",
        user.id
    );
}
