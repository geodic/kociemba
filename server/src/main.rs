use axum::Json;
use axum::{
    extract::Path,
    http::{HeaderValue, Method},
    response::Html,
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;

use kociemba::scramble;
use kociemba::solver::{self, SoutionResult};

#[tokio::main]
async fn main() {
    // build our application with a route
    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET]);
    let app = Router::new()
        .route("/", get(index))
        .route("/solve/:puzzle", get(move |p| solve(p)))
        .route("/scramble", get(scramble))
        .layer(cors);

    let app = app.fallback(index);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:32125")
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html("<p>Solve a cube: http://localhost:32125/solve/<Facelet String></p>
    Example: <a href=\"http://localhost:32125/solve/DUUBULDBFRBFRRULLLBRDFFFBLURDBFDFDRFRULBLUFDURRBLBDUDL\">http://localhost:32125/solve/DUUBULDBFRBFRRULLLBRDFFFBLURDBFDFDRFRULBLUFDURRBLBDUDL</a>
    <p>Get a scramble: <a href=\"http://localhost:32125/scramble\">http://localhost:32125/scramble</a></p>")
}

async fn scramble() -> String {
    let ss = scramble::gen_scramble(25).unwrap();
    format!("Scramble: {}", scramble::scramble_to_str(&ss).unwrap())
}

async fn solve(Path(puzzle): Path<String>) -> Json<SoutionResult> {
    let result = solver::solve(&puzzle, 20, 3.0).unwrap();
    Json(result)
    // match result {
    //     Ok(solution) => Json(solution),
    //     error => Json(error),
    // }
}
