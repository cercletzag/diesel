use db_result::DbResult;

pub trait Row {
    fn take(&mut self) -> Option<&[u8]>;
}

pub struct DbRow<'a> {
    db_result: &'a DbResult,
    row_idx: usize,
    col_idx: usize,
}

impl<'a> DbRow<'a> {
    pub fn new(db_result: &'a DbResult, row_idx: usize) -> Self {
        DbRow {
            db_result: db_result,
            row_idx: row_idx,
            col_idx: 0,
        }
    }
}

impl<'a> Row for DbRow<'a> {
    fn take(&mut self) -> Option<&[u8]> {
        let current_idx = self.col_idx;
        self.col_idx += 1;
        self.db_result.get(self.row_idx, current_idx)
    }
}
