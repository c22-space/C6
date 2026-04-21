use worker::*;

mod auth;
mod jwt;
mod payments;
mod sync;
mod updates;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if req.method() == Method::Options {
        return cors(Response::empty()?);
    }

    let method = req.method();
    let path = req.path();

    let result = match (method.clone(), path.as_str()) {
        (Method::Get,  "/updates/check")      => updates::handle(req, &env).await,
        (Method::Post, "/auth/token")          => auth::handle_token(req, &env).await,
        (Method::Post, "/auth/refresh")        => auth::handle_refresh(req, &env).await,
        (Method::Post, "/auth/trial")          => auth::handle_trial(req, &env).await,
        (Method::Post, "/payments/webhook")    => payments::handle(req, &env).await,
        (Method::Get,  "/sync/pull")           => sync::handle_pull(req, &env).await,
        (Method::Post, "/sync/push")           => sync::handle_push(req, &env).await,
        (Method::Get,  "/sync/users")          => sync::handle_list_users(req, &env).await,
        (Method::Post, "/sync/users/invite")   => sync::handle_invite(req, &env).await,
        _ => {
            let segments: Vec<&str> = path.split('/').collect();
            if method == Method::Put
                && segments.len() == 5
                && segments[1] == "sync"
                && segments[2] == "users"
                && segments[4] == "role"
            {
                let id = segments[3].to_string();
                sync::handle_change_role(req, &env, id).await
            } else if method == Method::Delete
                && segments.len() == 4
                && segments[1] == "sync"
                && segments[2] == "users"
            {
                let id = segments[3].to_string();
                sync::handle_remove_user(req, &env, id).await
            } else {
                Response::error("Not found", 404)
            }
        }
    };

    match result {
        Ok(resp) => cors(resp),
        Err(e) => cors(Response::error(e.to_string(), 500)?),
    }
}

fn cors(mut resp: Response) -> Result<Response> {
    resp.headers_mut()
        .set("Access-Control-Allow-Origin", "*")?;
    resp.headers_mut().set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS",
    )?;
    resp.headers_mut().set(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization",
    )?;
    Ok(resp)
}
