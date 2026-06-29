use crate::core::registry::register_error::RegistryError;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};


type RegistryValue<T> = Arc<RwLock<T>>;

pub struct ClientRegistry<Key, T>
where
    Key: Eq + Hash,
{
    registry: RwLock<HashMap<Key, RegistryValue<T>>>,
}

impl<Key, T> Default for ClientRegistry<Key, T>
where
    Key: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Key, T> ClientRegistry<Key, T>
where
    Key: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            registry: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_item(&self, key: Key, value: T) -> Result<(), RegistryError> {
        
        let mut write_guard = self.get_write_lock()?;

        if write_guard.contains_key(&key) {
            return Err(RegistryError::KeyAlreadyExists(format!(
            "key already exists in the registry"
        )));
        }

        let value = self.create_registry_value(value);
        write_guard.insert(key, value);
        Ok(())
    }

    pub fn remove_item(&self, key: &Key) -> Result<(), RegistryError> {
        let mut write_guard = self.get_write_lock()?;
        
        let removed = write_guard.remove(key);
        if removed.is_none() {
            return Err(RegistryError::KeyNotFound(format!(
            "key not found in the registry"
        )));
        }

        Ok(())
    }

    pub fn contains_key(&self, key: &Key) -> Result<bool, RegistryError> {
        let read_guard = self.get_read_lock()?;
        Ok(read_guard.contains_key(key))
    }

    pub fn len(&self) -> Result<usize, RegistryError> {
        let read_guard = self.get_read_lock()?;
        Ok(read_guard.len())
    }

    pub fn is_empty(&self) -> Result<bool, RegistryError> {
        let read_guard = self.get_read_lock()?;
        Ok(read_guard.is_empty())
    }

    pub fn clear(&self) -> Result<(), RegistryError> {
        let mut write_guard = self.get_write_lock()?;
        write_guard.clear();
        Ok(())
    }

    pub fn get_item(&self, key: &Key) -> Result<Option<RegistryValue<T>>, RegistryError> {
        let read_guard = self.get_read_lock()?;
        let value = read_guard.get(key).cloned();
        Ok(value)
    }

    pub fn get_all_keys(&self) -> Result<Vec<Key>, RegistryError>
    where
        Key: Clone,
    {
        let read_guard = self.get_read_lock()?;
        let keys = read_guard.keys().cloned().collect();
        Ok(keys)
    }

    pub fn get_snapshot(&self) -> Result<Vec<RegistryValue<T>>, RegistryError>
    {
        let read_guard = self.get_read_lock()?;
        Ok(read_guard.values().cloned().collect())
    }

    fn get_write_lock(&self) -> Result<RwLockWriteGuard<'_, HashMap<Key, RegistryValue<T>>>, RegistryError> {
        self.registry
            .write()
            .map_err(|err| RegistryError::WriteLockPoisoned(err.to_string()))
    }

    fn get_read_lock(&self) -> Result<RwLockReadGuard<'_, HashMap<Key, RegistryValue<T>>>, RegistryError> {
        self.registry
            .read()
            .map_err(|err| RegistryError::ReadLockPoisoned(err.to_string()))
    }

    fn create_registry_value(&self, value: T) -> RegistryValue<T> {
        Arc::new(RwLock::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::ClientRegistry;
    use crate::core::registry::register_error::RegistryError;

    #[test]
    fn duplicate_key_insert_failure() {
        let registry = ClientRegistry::<u32, &'static str>::new();

        registry.add_item(7, "first").unwrap();
        let err = registry.add_item(7, "second").unwrap_err();

        assert!(matches!(err, RegistryError::KeyAlreadyExists(_)));
    }

    #[test]
    fn remove_missing_key_failure() {
        let registry = ClientRegistry::<u32, &'static str>::new();

        let err = registry.remove_item(&9).unwrap_err();

        assert!(matches!(err, RegistryError::KeyNotFound(_)));
    }

    #[test]
    fn get_existing_item_success() {
        let registry = ClientRegistry::<u32, &'static str>::new();

        registry.add_item(3, "value").unwrap();
        let value = registry.get_item(&3).unwrap().unwrap();

        assert_eq!(*value.read().unwrap(), "value");
    }

    #[test]
    fn remove_existing_key_success() {
        let registry = ClientRegistry::<u32, &'static str>::new();

        registry.add_item(11, "value").unwrap();
        registry.remove_item(&11).unwrap();

        assert_eq!(registry.contains_key(&11).unwrap(), false);
    }

    #[test]
    fn get_missing_item_returns_none_success() {
        let registry = ClientRegistry::<u32, &'static str>::new();

        let value = registry.get_item(&42).unwrap();

        assert!(value.is_none());
    }

    #[test]
    fn clear_registry_resets_length_success() {
        let registry = ClientRegistry::<u32, &'static str>::new();

        registry.add_item(1, "a").unwrap();
        registry.add_item(2, "b").unwrap();
        assert_eq!(registry.len().unwrap(), 2);

        registry.clear().unwrap();

        assert_eq!(registry.is_empty().unwrap(), true);
        assert_eq!(registry.len().unwrap(), 0);
    }

    #[test]
    fn get_all_keys_returns_inserted_keys_success() {
        let registry = ClientRegistry::<u32, &'static str>::new();

        registry.add_item(10, "a").unwrap();
        registry.add_item(20, "b").unwrap();

        let mut keys = registry.get_all_keys().unwrap();
        keys.sort();

        assert_eq!(keys, vec![10, 20]);
    }

    #[test]
    fn default_constructor_creates_empty_registry_success() {
        let registry = ClientRegistry::<u32, &'static str>::default();

        assert_eq!(registry.is_empty().unwrap(), true);
        assert_eq!(registry.len().unwrap(), 0);
    }

    #[test]
    fn get_snapshot_returns_all_inserted_values_success() {
        let registry = ClientRegistry::<u32, &'static str>::new();

        registry.add_item(1, "alpha").unwrap();
        registry.add_item(2, "beta").unwrap();

        let snapshot = registry.get_snapshot().unwrap();
        let mut values: Vec<&'static str> = snapshot
            .iter()
            .map(|entry| *entry.read().unwrap())
            .collect();
        values.sort();

        assert_eq!(values, vec!["alpha", "beta"]);
    }
}