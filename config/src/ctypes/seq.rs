use im::Vector;

use crate::ctypes::CType;

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CSeq {
    pub(crate) inner: Vector<CItem>,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) template: Box<CType>,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) name: Option<&'static str>,
}

impl CSeq {
    pub fn new(template: CType) -> Self {
        Self {
            inner: im::Vector::new(),
            template: Box::new(template),
            name: None,
        }
    }

    pub fn get(&self) -> &im::Vector<CItem> {
        &self.inner
    }
    pub fn get_mut(&mut self) -> &mut im::Vector<CItem> {
        &mut self.inner
    }

    pub fn push(&mut self, ty: CType) {
        self.inner.push_back(CItem::new(ty, self.inner.len()))
    }

    pub fn remove(&mut self, idx: usize) {
        self.inner.remove(idx);
        for (i, item) in self.inner.iter_mut().enumerate() {
            item.index = i;
        }
    }

    pub fn set(&mut self, vec: im::Vector<CItem>) {
        self.inner = vec;
    }
}

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CItem {
    pub index: usize,
    pub ty: CType,
}

impl CItem {
    pub fn new(ty: CType, idx: usize) -> Self {
        Self { index: idx, ty }
    }
}
