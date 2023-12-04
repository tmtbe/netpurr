#[derive(Default,Clone,Debug,PartialEq,Eq)]
pub struct HttpRecord {
    pub request: Request,
    pub response: Response,
}

#[derive(Default,Clone,Debug,PartialEq,Eq)]
pub struct Request {
    pub method: String,
    pub url: String,
}

#[derive(Default,Clone,Debug,PartialEq,Eq)]
pub struct Response {}