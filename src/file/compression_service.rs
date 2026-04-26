use crate::common::error::DomainError;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use tar::{Archive as TarArchive, Builder as TarBuilder};
use zip::{ZipArchive, ZipWriter, write::FileOptions};

pub enum CompressionFormat {
    Zip,
    TarGz,
}

pub struct CompressionService;

impl CompressionService {
    pub fn new() -> Self {
        Self
    }

    pub fn compress(
        &self,
        user_path: &Path,
        source_path: &str,
        format: CompressionFormat,
    ) -> Result<String, DomainError> {
        let full_source = user_path.join(source_path);
        if !full_source.exists() {
            return Err(DomainError::FileNotFound);
        }

        let archive_name = match format {
            CompressionFormat::Zip => format!("{}.zip", source_path),
            CompressionFormat::TarGz => format!("{}.tar.gz", source_path),
        };
        let full_archive_path = user_path.join(&archive_name);

        match format {
            CompressionFormat::Zip => self.create_zip(&full_source, &full_archive_path)?,
            CompressionFormat::TarGz => self.create_tar_gz(&full_source, &full_archive_path)?,
        }

        Ok(archive_name)
    }

    pub fn decompress(&self, user_path: &Path, archive_path: &str) -> Result<(), DomainError> {
        let full_archive = user_path.join(archive_path);
        if !full_archive.exists() {
            return Err(DomainError::FileNotFound);
        }

        let ext = archive_path.to_lowercase();
        if ext.ends_with(".zip") {
            self.extract_zip(&full_archive, user_path)
        } else if ext.ends_with(".tar.gz") || ext.ends_with(".tgz") {
            self.extract_tar_gz(&full_archive, user_path)
        } else if ext.ends_with(".rar") {
            self.extract_rar(&full_archive, user_path)
        } else {
            Err(DomainError::Internal("Unsupported archive format".into()))
        }
    }

    fn create_zip(&self, source: &Path, archive_path: &Path) -> Result<(), DomainError> {
        let file = File::create(archive_path).map_err(|e| DomainError::Io(e))?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        if source.is_dir() {
            self.add_dir_to_zip(&mut zip, source, source, options)?;
        } else {
            let file_name = source.file_name().unwrap().to_string_lossy();
            zip.start_file(file_name, options).map_err(|e| DomainError::Internal(e.to_string()))?;
            let mut f = File::open(source).map_err(|e| DomainError::Io(e))?;
            io::copy(&mut f, &mut zip).map_err(|e| DomainError::Io(e))?;
        }

        zip.finish().map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    fn add_dir_to_zip<W: Write + io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        base_path: &Path,
        current_path: &Path,
        options: FileOptions<()>,
    ) -> Result<(), DomainError> {
        for entry in fs::read_dir(current_path).map_err(|e| DomainError::Io(e))? {
            let entry = entry.map_err(|e| DomainError::Io(e))?;
            let path = entry.path();
            let name = path.strip_prefix(base_path).unwrap().to_string_lossy();

            if path.is_dir() {
                zip.add_directory(name, options).map_err(|e| DomainError::Internal(e.to_string()))?;
                self.add_dir_to_zip(zip, base_path, &path, options)?;
            } else {
                zip.start_file(name, options).map_err(|e| DomainError::Internal(e.to_string()))?;
                let mut f = File::open(path).map_err(|e| DomainError::Io(e))?;
                io::copy(&mut f, zip).map_err(|e| DomainError::Io(e))?;
            }
        }
        Ok(())
    }

    fn create_tar_gz(&self, source: &Path, archive_path: &Path) -> Result<(), DomainError> {
        let file = File::create(archive_path).map_err(|e| DomainError::Io(e))?;
        let enc = GzEncoder::new(file, Compression::default());
        let mut tar = TarBuilder::new(enc);

        if source.is_dir() {
            tar.append_dir_all(".", source).map_err(|e| DomainError::Io(e))?;
        } else {
            let file_name = source.file_name().unwrap().to_string_lossy();
            tar.append_path_with_name(source, file_name.as_ref()).map_err(|e| DomainError::Io(e))?;
        }

        tar.finish().map_err(|e| DomainError::Io(e))?;
        Ok(())
    }

    fn extract_zip(&self, archive_path: &Path, dest: &Path) -> Result<(), DomainError> {
        let file = File::open(archive_path).map_err(|e| DomainError::Io(e))?;
        let mut archive = ZipArchive::new(file).map_err(|e| DomainError::Internal(e.to_string()))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| DomainError::Internal(e.to_string()))?;
            let outpath = match file.enclosed_name() {
                Some(path) => dest.join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| DomainError::Io(e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).map_err(|e| DomainError::Io(e))?;
                    }
                }
                let mut outfile = File::create(&outpath).map_err(|e| DomainError::Io(e))?;
                io::copy(&mut file, &mut outfile).map_err(|e| DomainError::Io(e))?;
            }
        }
        Ok(())
    }

    fn extract_tar_gz(&self, archive_path: &Path, dest: &Path) -> Result<(), DomainError> {
        let file = File::open(archive_path).map_err(|e| DomainError::Io(e))?;
        let dec = GzDecoder::new(file);
        let mut archive = TarArchive::new(dec);
        archive.unpack(dest).map_err(|e| DomainError::Io(e))?;
        Ok(())
    }

    fn extract_rar(&self, archive_path: &Path, dest: &Path) -> Result<(), DomainError> {
        let path_str = archive_path.to_string_lossy().to_string();
        let mut archive = unrar::Archive::new(&path_str)
            .open_for_processing()
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        
        while let Some(header) = archive.read_header().map_err(|e| DomainError::Internal(e.to_string()))? {
            let is_file = header.entry().is_file();
            let filename = header.entry().filename.clone();
            archive = if is_file {
                header.extract_to(dest.join(&filename))
                    .map_err(|e| DomainError::Internal(e.to_string()))?
            } else {
                header.skip().map_err(|e| DomainError::Internal(e.to_string()))?
            };
        }
        Ok(())
    }
}
