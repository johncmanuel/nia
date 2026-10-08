use super::App;

impl App {
    pub(super) fn primary_view(&self, index: usize) -> usize {
        let Some(path) = self.tabs[index].document.path.as_ref() else {
            return index;
        };
        self.tabs
            .iter()
            .position(|tab| tab.document.path.as_ref() == Some(path))
            .unwrap_or(index)
    }

    pub(super) fn view_survives(&self, index: usize, closing: &[u64]) -> bool {
        let Some(path) = self.tabs[index].document.path.as_ref() else {
            return false;
        };
        self.tabs
            .iter()
            .any(|tab| tab.document.path.as_ref() == Some(path) && !closing.contains(&tab.id))
    }
}