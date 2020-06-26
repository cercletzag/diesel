use std::ffi::CStr;
use std::ptr::NonNull;
use std::slice;

use super::ffi;
use crate::mysql::connection::bind::Flags;

pub struct StatementMetadata {
    result: NonNull<ffi::MYSQL_RES>,
}

impl StatementMetadata {
    pub fn new(result: NonNull<ffi::MYSQL_RES>) -> Self {
        StatementMetadata { result }
    }

    pub fn fields(&'_ self) -> &'_ [MysqlFieldMetadata<'_>] {
        unsafe {
            let num_fields = ffi::mysql_num_fields(self.result.as_ptr());
            let field_ptr = ffi::mysql_fetch_fields(self.result.as_ptr());
            if field_ptr.is_null() {
                &[]
            } else {
                slice::from_raw_parts(field_ptr as _, num_fields as usize)
            }
        }
    }
}

impl Drop for StatementMetadata {
    fn drop(&mut self) {
        unsafe { ffi::mysql_free_result(self.result.as_mut()) };
    }
}

#[repr(transparent)]
pub struct MysqlFieldMetadata<'a>(ffi::MYSQL_FIELD, std::marker::PhantomData<&'a ()>);

impl<'a> MysqlFieldMetadata<'a> {
    pub fn field_name(&self) -> Option<&str> {
        if self.0.name.is_null() {
            None
        } else {
            unsafe { CStr::from_ptr(self.0.name).to_str().ok() }
        }
    }

    pub fn field_type(&self) -> ffi::enum_field_types {
        self.0.type_
    }

    pub(crate) fn flags(&self) -> Flags {
        Flags::from(self.0.flags)
    }
}
