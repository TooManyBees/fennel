use crate::util::HasKeywords;
use intrusive_collections::{linked_list::LinkedListOps, Adapter, LinkedList, PointerOps};

pub fn pluck_item_from_list<A>(
    list: &mut LinkedList<A>,
    keyword: &str,
) -> Option<<<A as Adapter>::PointerOps as PointerOps>::Pointer>
where
    A: Adapter,
    A::LinkOps: LinkedListOps,
    <<A as Adapter>::PointerOps as PointerOps>::Value: HasKeywords,
{
    let mut found = None;

    'obj_list: for obj in list.iter() {
        for kw in obj.keywords() {
            let is_match = kw.starts_with(keyword);
            let exact = is_match && kw.len() == keyword.len();
            if exact {
                found = Some(obj);
                break 'obj_list;
            }
            if is_match && found.is_none() {
                found = Some(obj);
                break;
            }
        }
    }

    if let Some(obj) = found {
        let ptr = obj as *const <<A as Adapter>::PointerOps as PointerOps>::Value;
        let mut cursor = unsafe { list.cursor_mut_from_ptr(ptr) };
        let gotten = cursor.remove().unwrap();
        return Some(gotten);
    }

    None
}
