pub struct Request {
    tenant_id: u16,
    start: u64,
}

impl Request {
    pub fn new(tenant: u16, rdstc: u64) -> Request {
        Request {
            tenant_id: tenant,
            start: rdstc,
        }
    }

    pub fn run(&self, now: u64) -> u64 {
        now - self.start
    }

    pub fn get_tenant(&self) -> u16 {
        self.tenant_id.clone()
    }
}
