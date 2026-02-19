pub enum IoResult {
    Success(usize),
    Closed,
    Timeout,
}

impl IoResult {
    pub fn is_success(&self) -> bool {
        matches!(self, IoResult::Success(_))
    }
    
    pub fn is_closed(&self) -> bool {
        matches!(self, IoResult::Closed)
    }
    
    pub fn is_timeout(&self) -> bool {
        matches!(self, IoResult::Timeout)
    }
    
    pub fn bytes(&self) -> Option<usize> {
        if let IoResult::Success(n) = self {
            Some(*n)
        } else {
            None
        }
    }
}

impl std::fmt::Display for IoResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoResult::Success(n) => write!(f, "Success({} bytes)", n),
            IoResult::Closed => write!(f, "Closed"),
            IoResult::Timeout => write!(f, "Timeout"),
        }
    }
}