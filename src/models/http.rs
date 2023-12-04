#[derive(Clone)]
pub(crate) struct HttpRecord {
    pub(crate) request: Request,
    pub(crate) response: Response,
}

#[derive(Clone)]
pub(crate) struct Request {
    pub(crate) method: String,
    pub(crate) url: String,
}

#[derive(Clone)]
pub(crate) struct Response {}