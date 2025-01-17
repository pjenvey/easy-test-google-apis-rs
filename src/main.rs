extern crate google_spanner1 as spanner1;
extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
use oauth2::ServiceAccountAccess;
use spanner1::Spanner;
use spanner1::Error;
use yup_oauth2::service_account_key_from_file;

use futures::future::lazy;
use futures::future::Future;
use futures::future;
use tokio_threadpool::ThreadPool;

const DATABASE_INSTANCE: &'static str = "projects/lustrous-center-242019/instances/testing1";

fn do_a_blocking_thing() -> Box<Future<Item = usize, Error = ()> + Send> {
    let secret = service_account_key_from_file(&String::from("service-account.json")).unwrap();
    let client = hyper::Client::with_connector(hyper::net::HttpsConnector::new(
        hyper_rustls::TlsClient::new(),
    ));
    let mut access = ServiceAccountAccess::new(secret, client);
    use yup_oauth2::GetToken;
    println!(
        "{:?}",
        access
            .token(&vec!["https://www.googleapis.com/auth/spanner.data"])
            .unwrap()
    );
    let client2 = hyper::Client::with_connector(hyper::net::HttpsConnector::new(
        hyper_rustls::TlsClient::new(),
    ));
    let hub = Spanner::new(client2, access);
    let result = hub
        .projects()
        .instances_databases_list(DATABASE_INSTANCE)
        .doit();

    let rv = match result {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
            Error::HttpError(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => {
                println!("{}", e);
                Box::new(future::err(()))
            }
        },
        Ok(res) => match res.1.databases {
            Some(dbs) => {
                println!("{:?}", dbs);
                Box::new(future::ok(dbs.len()))
            }
            None => {
                println!("no databases");
                Box::new(future::err(()))
            }
        },
    };
    rv
}

fn main() {
    let pool = ThreadPool::new();
    let fut = pool.spawn_handle(lazy(move || {
        println!("hello from another thread");
        do_a_blocking_thing()
    }));
    println!("hello world");
    println!("future result {}", fut.wait().unwrap());

}
