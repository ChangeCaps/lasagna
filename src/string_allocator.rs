use dashmap::DashSet;

lazy_static::lazy_static! {
    pub static ref GLOBAL_ALLOCATOR: StringAllocator = StringAllocator::default();
}

#[derive(Default)]
pub struct StringAllocator {
    strings: DashSet<&'static str>,
}

impl StringAllocator {
    pub fn alloc(&self, string: &str) -> &'static str {
        if !self.strings.contains(string) {
            let string = Box::leak(String::from(string).into_boxed_str());

            self.strings.insert(string);
        }

        *self.strings.get(string).unwrap()
    }
}

#[inline]
pub fn static_str(string: &str) -> &'static str {
    GLOBAL_ALLOCATOR.alloc(string)
}
