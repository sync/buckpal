use tide::{Body, Error, Response, StatusCode};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct SendMoneyResponse {
    message: String,
}

pub fn success_to_res(message: &str) -> tide::Result<Response> {
    let send_money_response = SendMoneyResponse {
        message: String::from(message),
    };

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(Body::from_json(&send_money_response)?);

    Ok(res)
}

#[allow(dead_code)]
pub fn err_to_res(err: Error) -> tide::Result<Response> {
    let send_money_response = SendMoneyResponse {
        message: format!("Unable to process request: {}", err.to_string()),
    };

    let mut res = Response::new(err.status());
    res.set_body(Body::from_json(&send_money_response)?);

    Ok(res)
}
