use super::request::Request;
use std::collections::VecDeque;

pub struct Tenant {
    // Task runqueue for this tenant.
    pub rq: VecDeque<Request>,

    // The ID of the current tenant.
    pub tenant_id: u16,
}

impl Tenant {
    pub fn new(tenant: u16) -> Tenant {
        Tenant {
            rq: VecDeque::with_capacity(32),
            tenant_id: tenant,
        }
    }

    pub fn add_request(&mut self, rdtsc: u64) {
        let req = Request::new(self.tenant_id, rdtsc);
        self.rq.push_back(req);
    }

    pub fn get_request(&mut self) -> Option<Request> {
        self.rq.pop_front()
    }
}
