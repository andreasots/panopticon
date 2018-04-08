use futures::IntoFuture;
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::State;
use hyper::mime::{self, Mime};
use hyper::StatusCode;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::Path;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct PathParams {
    file: String,
}

fn read_file(path: &Path) -> Result<Vec<u8>, Error> {
    let mut file = File::open(path)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;
    Ok(content)
}

fn guess_mime_type(path: &Path) -> Mime {
    match path.extension().and_then(OsStr::to_str) {
        Some("css") => mime::TEXT_CSS,
        Some("jpg") => mime::IMAGE_JPEG,
        Some("js") => mime::TEXT_JAVASCRIPT,
        Some("png") => mime::IMAGE_PNG,
        Some("svg") => "image/svg+xml".parse().unwrap(),
        _ => mime::APPLICATION_OCTET_STREAM,
    }
}

pub fn static_files(state: State) -> Box<HandlerFuture> {
    let path = {
        let params = state.borrow::<PathParams>();
        Path::new("static/").join(&params.file)
    };

    let ret = match read_file(&path) {
        Ok(content) => {
            let response = create_response(
                &state,
                StatusCode::Ok,
                Some((content, guess_mime_type(&path))),
            );

            Ok((state, response))
        }
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                Err((
                    state,
                    err.into_handler_error().with_status(StatusCode::NotFound),
                ))
            } else {
                Err((state, err.into_handler_error()))
            }
        }
    };

    Box::new(ret.into_future())
}
