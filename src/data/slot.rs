#[derive(Debug, Clone, PartialEq)]
pub struct DataComponent {
    pub component_type: i32,
    pub raw_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub count: i32,
    pub item_id: i32,
    pub components_to_add: Vec<DataComponent>,
    pub components_to_remove: Vec<i32>,
}

pub type Slot = Option<ItemStack>;
