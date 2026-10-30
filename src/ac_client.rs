use std::mem::size_of;

use crate::esp::Entity;
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, VirtualQuery,
};

pub const PLAYER_COUNT: u32 = 0x18AC0C;
pub const VIEW_MATRIX: u32 = 0x17DFD0;
pub const ENTITY_LIST: u32 = 0x18AC04;
pub const ENTITY_HEAD_POSITION: u32 = 0x4;
pub const ENTITY_FEET_POSITION: u32 = 0x28;
pub const ENTITY_HEALTH: u32 = 0xEC;
pub const MAX_ENTITIES: u32 = 32;

const ENTITY_READ_SIZE: u32 = ENTITY_HEALTH + size_of::<i32>() as u32;

pub fn read<T>(base_addr: u32, offset: u32) -> Option<T>
where
    T: Copy,
{
    let address = base_addr.checked_add(offset)?;

    readable(address, size_of::<T>() as u32).then(|| unsafe { read_unchecked(address) })
}

pub fn read_entity(entity_list_base_addr: u32, index: u32) -> Option<Entity> {
    let base_addr = read::<u32>(entity_list_base_addr, index * 4)?;

    if !readable(base_addr, ENTITY_READ_SIZE) {
        return None;
    }

    unsafe {
        Some(Entity {
            health: read_unchecked(base_addr + ENTITY_HEALTH),
            head: read_unchecked::<[f32; 3]>(base_addr + ENTITY_HEAD_POSITION).into(),
            feet: read_unchecked::<[f32; 3]>(base_addr + ENTITY_FEET_POSITION).into(),
        })
    }
}

unsafe fn read_unchecked<T>(address: u32) -> T
where
    T: Copy,
{
    unsafe { *(address as *const T) }
}

fn readable(address: u32, size: u32) -> bool {
    let mut current = address as usize;
    let end = match current.checked_add(size as usize) {
        Some(end) => end,
        None => return false,
    };

    while current < end {
        let mut info = MEMORY_BASIC_INFORMATION::default();
        let result = unsafe {
            VirtualQuery(
                Some(current as *const _),
                &mut info,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };

        if result == 0 || info.State != MEM_COMMIT {
            return false;
        }

        let protect = info.Protect.0;

        if protect & PAGE_NOACCESS.0 != 0 || protect & PAGE_GUARD.0 != 0 {
            return false;
        }

        let region_end = (info.BaseAddress as usize).saturating_add(info.RegionSize);

        if region_end <= current {
            return false;
        }

        current = region_end.min(end);
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_valid_entity() {
        let mut buffer = vec![0u8; 0x200];
        let base = buffer.as_mut_ptr() as usize;
        let entity = base + 0x100;

        unsafe {
            *(buffer.as_mut_ptr() as *mut u32) = entity as u32;
            *((entity + ENTITY_HEALTH as usize) as *mut i32) = 100;
            *((entity + ENTITY_HEAD_POSITION as usize) as *mut [f32; 3]) = [1.0, 2.0, 3.0];
            *((entity + ENTITY_FEET_POSITION as usize) as *mut [f32; 3]) = [4.0, 5.0, 6.0];
        }

        let result = read_entity(base as u32, 0).unwrap();

        assert_eq!(result.health, 100);
        assert_eq!(
            (result.head.x, result.head.y, result.head.z),
            (1.0, 2.0, 3.0)
        );
        assert_eq!(
            (result.feet.x, result.feet.y, result.feet.z),
            (4.0, 5.0, 6.0)
        );
    }

    #[test]
    fn rejects_an_unmapped_address() {
        assert!(read::<u32>(0, 0).is_none());
        assert!(read_entity(0, 0).is_none());
    }
}
