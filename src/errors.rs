use std::backtrace::Backtrace;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Uri};
use phf::phf_map;

/// Default http error messages
pub static MESSAGES: phf::Map<u16, &'static str> = phf_map! {
    100u16 => "Continue",
    101u16 => "Switching protocols",
    102u16 => "Processing",
    103u16 => "Early Hints",

    200u16 => "OK",
    201u16 => "Created",
    202u16 => "Accepted",
    203u16 => "Non-Authoritative Information",
    204u16 => "No Content",
    205u16 => "Reset Content",
    206u16 => "Partial Content",
    207u16 => "Multi-Status",
    208u16 => "Already Reported",
    226u16 => "IM Used",

    300u16 => "Multiple Choices",
    301u16 => "Moved Permanently",
    302u16 => "Found (Previously \"Moved Temporarily\")",
    303u16 => "See Other",
    304u16 => "Not Modified",
    305u16 => "Use Proxy",
    306u16 => "Switch Proxy",
    307u16 => "Temporary Redirect",
    308u16 => "Permanent Redirect",

    400u16 => "Bad Request",
    401u16 => "Unauthorized",
    402u16 => "Payment Required",
    403u16 => "Forbidden",
    404u16 => "Not Found",
    405u16 => "Method Not Allowed",
    406u16 => "Not Acceptable",
    407u16 => "Proxy Authentication Required",
    408u16 => "Request Timeout",
    409u16 => "Conflict",
    410u16 => "Gone",
    411u16 => "Length Required",
    412u16 => "Precondition Failed",
    413u16 => "Payload Too Large",
    414u16 => "URI Too Long",
    415u16 => "Unsupported Media Type",
    416u16 => "Range Not Satisfiable",
    417u16 => "Expectation Failed",
    418u16 => "I'm a Teapot",
    421u16 => "Misdirected Request",
    422u16 => "Unprocessable Entity",
    423u16 => "Locked",
    424u16 => "Failed Dependency",
    425u16 => "Too Early",
    426u16 => "Upgrade Required",
    428u16 => "Precondition Required",
    429u16 => "Too Many Requests",
    431u16 => "Request Header Fields Too Large",
    451u16 => "Unavailable For Legal Reasons",

    500u16 => "Internal Server Error",
    501u16 => "Not Implemented",
    502u16 => "Bad Gateway",
    503u16 => "Service Unavailable",
    504u16 => "Gateway Timeout",
    505u16 => "HTTP Version Not Supported",
    506u16 => "Variant Also Negotiates",
    507u16 => "Insufficient Storage",
    508u16 => "Loop Detected",
    510u16 => "Not Extended",
    511u16 => "Network Authentication Required",
};

pub fn default_error_page(
    code: &u16,
    reason: &String,
    method: &Method,
    uri: &Uri,
    body: String,
) -> hyper::Response<Full<Bytes>> {
    #[cfg(debug_assertions)]
    let styles = r#"
*{box-sizing:border-box}body{padding:.5rem;margin:0;min-height:100vh;min-height:100dvh;display:flex;justify-content:center;align-items:center}#overlay{color:#000;border:1px solid #9e9e9e;background:#b8b6b6;display:flex;flex-direction:column;width:97%;min-height:95vh;min-height:95dvh;height:95%;border-radius:.5rem;box-shadow:rgba(0,0,0,0.25) 0 54px 55px,rgba(0,0,0,0.12) 0 -12px 30px,rgba(0,0,0,0.12) 0 4px 6px,rgba(0,0,0,0.17) 0 12px 13px,rgba(0,0,0,0.09) 0 -3px 5px}h1{font-size:2.65rem;text-align:center;margin:.5rem}h2{font-size:2.441rem}h3{font-size:1.953rem}h4{font-size:1.563rem}h5{font-size:1.25rem}small,.text_small{font-size:.8rem}details summary{cursor:pointer}hr{border:1px solid rgba(0,0,0,0.5)}details summary>*{display:inline}summary{background-color:rgba(200,15,50,0.5);padding-block:.25rem;padding-inline:.5rem;font-weight:700}summary::marker{color:rgba(200,15,50,0.50)}details{border:1px solid rgba(200,15,50,0.75);border-radius:.25rem;display:flex;gap:.5rem;width:85%;margin-inline:auto;margin-block:1rem;font-family:Arial,sans-serif;font-size:1.1rem}details>#body{background-color:rgba(200,15,50,0.25);padding:1rem;display:flex;flex-direction:column;gap:.5rem}.path{background-color:rgba(0,0,0,.5);padding:.2rem .35rem;border-radius:.2rem}details>#body>div{width:80%;color:#fff;max-width:95ch;margin-inline:auto;border:1px solid rgba(0,0,0,.5);background-color:rgba(0,0,0,.25);display:flex;flex-wrap:wrap}details>#body>div>span:first-child{display:inline-block;background:#000;padding:.5rem;width:40%;display:flex;align-items:center;justify-content:center}details>#body>div>span:last-child{display:inline-block;text-align:center;padding:.5rem;width:60%;max-height:6rem;overflow:auto}details>#body>div>div:first-child{display:inline-block;text-align:center;background:#000;padding:.5rem;width:100%;max-height:15rem;overflow-y:auto}details>#body>div>pre{padding:1rem;width:100%;overflow:auto;max-height:20rem}table{color:#fff;width:100%;border:1px solid #000;border-collapse:collapse}thead{background:#000}tbody{padding:.5rem;background-color:rgba(0,0,0,.25)}td{padding-block:.5rem;text-align:center}#trace{border:1px solid rgba(200,15,50,0.75);box-sizing:border-box;border-radius:.25rem;height:100%;max-height:27rem;width:85%;margin-inline:auto;overflow:auto;background-color:rgba(200,15,50,0.25)}@media(prefers-color-scheme: dark){#overlay{background:#1c1c1c;border:1px solid #171717;color:#fff}details>#body>div>div:last-child{color:#fff}html{background:#333}}
    "#;

    #[cfg(debug_assertions)]
    return hyper::Response::builder()
        .status(code.clone())
        .header("Wayfinder-Reason", reason)
        .header("Content-Type", "text/html")
        .body(Full::new(Bytes::from(html_to_string_macro::html! {
        <!DOCTYPE html>
        <html lang="en">

        <head>
            <meta charset="UTF-8"/>
            <meta name="viewport" content="width=device-width, initial-scale=1"/>
            <style>
                {styles}
            </style>
        </head>

        <body>
            <div id="overlay">
                <h1>{code}" "{MESSAGES.get(&code).unwrap()}</h1>
                <details open>
                    <summary>
                        <h4>"Unhandled Error:"</h4>
                    </summary>
                    <div id="body">
                        <strong>{reason}</strong>
                        <div><span>"Method"</span><span>{method}</span></div>
                        <div><span>"Status"</span><span>{code}</span></div>
                        <div><span>"URI"</span><span><span class="path">{uri.path()}</span></span></div>
                        <div><span>"Query"</span><span>{uri.query().unwrap_or("")}</span></div>
                        <div>
                            <div>"Body"</div>
                            <div>{body}</div>
                        </div>
                    </div>
                </details>
                <div id="trace">
                    <pre>
        {Backtrace::capture().to_string().replace("<", "&lt;").replace(">", "&gt;")}
                    </pre>
                </div>
            </div>
        </body>

        </html>
                })))
        .unwrap();

    #[cfg(not(debug_assertions))]
    return hyper::Response::builder()
        .status(code.clone())
        .header("Wayfinder-Reason", reason)
        .body(Full::new(Bytes::new()))
        .unwrap();
}
