// LDC - Local Data Cache
use std::path::PathBuf;
pub fn cache_file_string(path: &str) -> CacheFile<String> {
    let file_handler = FileHandler::new(path.to_string());
    CacheFile::<String>::new(file_handler)
}
pub fn cache_file_integer(path: &str) -> CacheFile<i32> {
    let file_handler = FileHandler::new(path.to_string());
    CacheFile::<i32>::new(file_handler)
}
pub fn cache_file_float(path: &str) -> CacheFile<f64> {
    let file_handler = FileHandler::new(path.to_string());
    CacheFile::<f64>::new(file_handler)
}
pub fn cache_file_bool(path: &str) -> CacheFile<bool> {
    let file_handler = FileHandler::new(path.to_string());
    CacheFile::<bool>::new(file_handler)
}
pub fn cache_folder(path: &str) -> PathBuf {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        std::fs::create_dir_all(&path_buf).unwrap();
    }
    path_buf
}
pub struct CacheFile<T> {
    file_handler: FileHandler,
    data_type: T,
}

impl<T> CacheFile<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(file_handler: FileHandler) -> Self
    where
        T: Default,
    {
        let mut file_handler = file_handler;
        let data_type = bincode::deserialize(&file_handler.read().unwrap_or(vec![]))
            .unwrap_or_else(|_| T::default());
        Self {
            file_handler,
            data_type,
        }
    }

    pub fn get_data(&self) -> &T {
        &self.data_type
    }

    pub fn get_mut_data(&mut self) -> &mut T {
        &mut self.data_type
    }
    pub fn read(&mut self) -> Result<T, Box<dyn std::error::Error>>
    where
        T: Clone,
    {
        let bytes = self.file_handler.read()?;
        self.data_type = bincode::deserialize(&bytes)?;
        Ok(self.data_type.clone())
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = bincode::serialize(&self.data_type)?;
        self.file_handler.write(&bytes)
    }

    pub fn append(&mut self, data_type: &T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: std::fmt::Display + std::str::FromStr + Clone,
        <T as std::str::FromStr>::Err: std::error::Error + 'static,
    {
        let current_data = self.read()?;
        let mut current_str = current_data.to_string();
        current_str.push_str(&data_type.to_string());
        self.data_type = current_str.parse()?;
        self.write()
    }

    pub fn delete(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.file_handler.delete()
    }

    pub fn exists(&self) -> bool {
        self.file_handler.exists()
    }
}
pub struct CacheConfig<T> {
    file_handler: FileHandler,
    config: T,
}

impl<T> CacheConfig<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(file_handler: FileHandler) -> Self
    where
        T: Default,
    {
        let mut file_handler = file_handler;
        let config = serde_json::from_slice(&file_handler.read().unwrap_or(vec![]))
            .unwrap_or_else(|_| T::default());
        Self {
            file_handler,
            config,
        }
    }

    pub fn get_config(&self) -> &T {
        &self.config
    }

    pub fn get_mut_config(&mut self) -> &mut T {
        &mut self.config
    }

    pub fn read(&mut self) -> Result<T, Box<dyn std::error::Error>>
    where
        T: Clone,
    {
        let bytes = self.file_handler.read()?;
        self.config = serde_json::from_slice(&bytes)?;
        Ok(self.config.clone())
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json_str = serde_json::to_string(&self.config)?;
        self.file_handler.write(json_str.as_bytes())
    }

    pub fn delete(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.file_handler.delete()
    }

    pub fn exists(&self) -> bool {
        self.file_handler.exists()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_string_cache() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("string.txt");
        let file_handler = FileHandler::new(file_path.to_str().unwrap().to_string());
        let mut cache = CacheFile::<String>::new(file_handler);

        *cache.get_mut_data() = String::from("test");
        cache.write().unwrap();
        assert_eq!(cache.read().unwrap(), "test");
        cache.delete().unwrap();
        dir.close().unwrap();
    }

    #[test]
    fn test_integer_cache() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("int.txt");
        let file_handler = FileHandler::new(file_path.to_str().unwrap().to_string());
        let mut cache = CacheFile::<i32>::new(file_handler);

        *cache.get_mut_data() = 42;
        cache.write().unwrap();
        assert_eq!(cache.read().unwrap(), 42);
        cache.delete().unwrap();
        dir.close().unwrap();
    }

    #[test]
    fn test_float_cache() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("float.txt");
        let file_handler = FileHandler::new(file_path.to_str().unwrap().to_string());
        let mut cache = CacheFile::<f64>::new(file_handler);

        *cache.get_mut_data() = 3.14;
        cache.write().unwrap();
        assert_eq!(cache.read().unwrap(), 3.14);
        cache.delete().unwrap();
        dir.close().unwrap();
    }

    #[test]
    fn test_bool_cache() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("bool.txt");
        let file_handler = FileHandler::new(file_path.to_str().unwrap().to_string());
        let mut cache = CacheFile::<bool>::new(file_handler);

        *cache.get_mut_data() = true;
        cache.write().unwrap();
        assert_eq!(cache.read().unwrap(), true);
        cache.delete().unwrap();
        dir.close().unwrap();
    }
}

// FileHandler
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileHandler {
    path: PathBuf,
    content: Option<Vec<u8>>,
}

impl FileHandler {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: PathBuf::from(path.as_ref()),
            content: None,
        }
    }

    pub fn read(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;
        self.content = Some(content.clone());
        Ok(content)
    }

    pub fn write(&self, content: &[u8]) -> Result<(), Box<dyn Error>> {
        let file = File::create(&self.path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(content)?;
        Ok(())
    }

    pub fn append(&self, content: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(content)?;
        Ok(())
    }

    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        fs::remove_file(&self.path)?;
        Ok(())
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn copy_to<P: AsRef<Path>>(&self, destination: P) -> Result<(), Box<dyn Error>> {
        fs::copy(&self.path, destination)?;
        Ok(())
    }

    pub fn move_to<P: AsRef<Path>>(&self, destination: P) -> Result<(), Box<dyn Error>> {
        fs::rename(&self.path, destination)?;
        Ok(())
    }

    pub fn metadata(&self) -> Result<fs::Metadata, Box<dyn Error>> {
        Ok(fs::metadata(&self.path)?)
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }
}
