// LDC - Local Data Cache
use std::path::PathBuf;
use tracing::{debug, error, info, instrument, warn};

#[instrument]
pub fn cache_file_string(path: &str) -> CacheFile<String> {
    debug!("Creating string cache file at {}", path);
    let file_handler = FileHandler::new(path.to_string());
    CacheFile::<String>::new(file_handler)
}

#[instrument]
pub fn cache_file_integer(path: &str) -> CacheFile<i32> {
    debug!("Creating integer cache file at {}", path);
    let file_handler = FileHandler::new(path.to_string());
    CacheFile::<i32>::new(file_handler)
}

#[instrument]
pub fn cache_file_float(path: &str) -> CacheFile<f64> {
    debug!("Creating float cache file at {}", path);
    let file_handler = FileHandler::new(path.to_string());
    CacheFile::<f64>::new(file_handler)
}

#[instrument]
pub fn cache_file_bool(path: &str) -> CacheFile<bool> {
    debug!("Creating boolean cache file at {}", path);
    let file_handler = FileHandler::new(path.to_string());
    CacheFile::<bool>::new(file_handler)
}

#[instrument]
pub fn cache_folder(path: &str) -> PathBuf {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        info!("Creating cache directory at {}", path);
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
    #[instrument(skip(file_handler))]
    pub fn new(file_handler: FileHandler) -> Self
    where
        T: Default,
    {
        debug!(
            "Initializing new cache file at {:?}",
            file_handler.get_path()
        );
        let mut file_handler = file_handler;
        let data_type = match file_handler.read() {
            Ok(bytes) => match bincode::deserialize(&bytes) {
                Ok(data) => {
                    debug!("Successfully loaded existing data");
                    data
                }
                Err(e) => {
                    warn!("Failed to deserialize existing data: {}", e);
                    T::default()
                }
            },
            Err(e) => {
                warn!("Failed to read file: {}", e);
                T::default()
            }
        };
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

    #[instrument(skip(self))]
    pub fn read(&mut self) -> Result<T, Box<dyn std::error::Error>>
    where
        T: Clone,
    {
        debug!("Reading from cache file");
        let bytes = self.file_handler.read()?;
        match bincode::deserialize(&bytes) {
            Ok(data) => {
                self.data_type = data;
                debug!("Successfully read data from cache");
                Ok(self.data_type.clone())
            }
            Err(e) => {
                error!("Failed to deserialize cache data: {}", e);
                Err(Box::new(e))
            }
        }
    }

    #[instrument(skip(self))]
    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Writing to cache file");
        match bincode::serialize(&self.data_type) {
            Ok(bytes) => {
                self.file_handler.write(&bytes)?;
                debug!("Successfully wrote data to cache");
                Ok(())
            }
            Err(e) => {
                error!("Failed to serialize data: {}", e);
                Err(Box::new(e))
            }
        }
    }

    #[instrument(skip(self))]
    pub fn append(&mut self, data_type: &T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: std::fmt::Display + std::str::FromStr + Clone + std::fmt::Debug,
        <T as std::str::FromStr>::Err: std::error::Error + 'static,
    {
        debug!("Appending data to cache file");
        let current_data = self.read()?;
        let mut current_str = current_data.to_string();
        current_str.push_str(&data_type.to_string());
        match current_str.parse() {
            Ok(parsed) => {
                self.data_type = parsed;
                self.write()?;
                debug!("Successfully appended data to cache");
                Ok(())
            }
            Err(e) => {
                error!("Failed to parse appended data: {}", e);
                Err(Box::new(e))
            }
        }
    }

    #[instrument(skip(self))]
    pub fn delete(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Deleting cache file");
        self.file_handler.delete()?;
        debug!("Successfully deleted cache file");
        Ok(())
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
    #[instrument(skip(file_handler))]
    pub fn new(file_handler: FileHandler) -> Self
    where
        T: Default,
    {
        debug!(
            "Initializing new config file at {:?}",
            file_handler.get_path()
        );
        let mut file_handler = file_handler;
        let config = match file_handler.read() {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(config) => {
                    debug!("Successfully loaded existing config");
                    config
                }
                Err(e) => {
                    warn!("Failed to deserialize config: {}", e);
                    T::default()
                }
            },
            Err(e) => {
                warn!("Failed to read config file: {}", e);
                T::default()
            }
        };
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

    #[instrument(skip(self))]
    pub fn read(&mut self) -> Result<T, Box<dyn std::error::Error>>
    where
        T: Clone,
    {
        debug!("Reading config file");
        let bytes = self.file_handler.read()?;
        match serde_json::from_slice(&bytes) {
            Ok(config) => {
                self.config = config;
                debug!("Successfully read config");
                Ok(self.config.clone())
            }
            Err(e) => {
                error!("Failed to deserialize config: {}", e);
                Err(Box::new(e))
            }
        }
    }

    #[instrument(skip(self))]
    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Writing config file");
        match serde_json::to_string(&self.config) {
            Ok(json_str) => {
                self.file_handler.write(json_str.as_bytes())?;
                debug!("Successfully wrote config");
                Ok(())
            }
            Err(e) => {
                error!("Failed to serialize config: {}", e);
                Err(Box::new(e))
            }
        }
    }

    #[instrument(skip(self))]
    pub fn delete(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Deleting config file");
        self.file_handler.delete()?;
        debug!("Successfully deleted config file");
        Ok(())
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

    #[instrument(skip(self))]
    pub fn read(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        debug!("Reading file: {:?}", self.path);
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;
        self.content = Some(content.clone());
        debug!("Successfully read {} bytes", content.len());
        Ok(content)
    }

    #[instrument(skip(self, content))]
    pub fn write(&self, content: &[u8]) -> Result<(), Box<dyn Error>> {
        debug!("Writing {} bytes to {:?}", content.len(), self.path);
        let file = File::create(&self.path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(content)?;
        debug!("Successfully wrote data");
        Ok(())
    }

    #[instrument(skip(self, content))]
    pub fn append(&self, content: &[u8]) -> Result<(), Box<dyn Error>> {
        debug!("Appending {} bytes to {:?}", content.len(), self.path);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(content)?;
        debug!("Successfully appended data");
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        debug!("Deleting file: {:?}", self.path);
        fs::remove_file(&self.path)?;
        debug!("Successfully deleted file");
        Ok(())
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    #[instrument(skip(self))]
    pub fn copy_to<P: AsRef<Path> + std::fmt::Debug>(
        &self,
        destination: P,
    ) -> Result<(), Box<dyn Error>> {
        debug!("Copying {:?} to {:?}", self.path, destination.as_ref());
        fs::copy(&self.path, destination)?;
        debug!("Successfully copied file");
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn move_to<P: AsRef<Path> + std::fmt::Debug>(
        &self,
        destination: P,
    ) -> Result<(), Box<dyn Error>> {
        debug!("Moving {:?} to {:?}", self.path, destination.as_ref());
        fs::rename(&self.path, destination)?;
        debug!("Successfully moved file");
        Ok(())
    }

    pub fn metadata(&self) -> Result<fs::Metadata, Box<dyn Error>> {
        Ok(fs::metadata(&self.path)?)
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }
}
