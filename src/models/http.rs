#[derive(Clone,Debug)]
pub struct HttpRecord {
    pub request: Request,
    pub response: Response,
}

#[derive(Clone,Debug)]
pub struct Request {
    pub method: String,
    pub url: String,
}

#[derive(Clone,Debug)]
pub struct Response {}