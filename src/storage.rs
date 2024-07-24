// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
type Result<T> = std::result::Result<T, ic_agent::AgentError>;

pub type FileId1 = String;
pub type Chunk = serde_bytes::ByteBuf;
pub type Err1 = String;
pub type Result8 = candid::MotokoResult<Chunk, Err1>;
pub type Result6 = candid::MotokoResult<(), Err1>;
pub type FileId2 = String;
#[derive(CandidType, Deserialize)]
pub struct FileMeta {
    pub id: FileId2,
    pub owners: Vec<Principal>,
    pub path: String,
    #[serde(rename = "chunkCount")]
    pub chunk_count: u128,
}
pub type Result7 = candid::MotokoResult<FileMeta, Err1>;
#[derive(CandidType, Deserialize)]
pub struct StorageStats1 {
    #[serde(rename = "fileCount")]
    pub file_count: candid::Nat,
    #[serde(rename = "cyclesBalance")]
    pub cycles_balance: candid::Nat,
    #[serde(rename = "memorySize")]
    pub memory_size: candid::Nat,
}

pub struct Service<'a>(pub Principal, pub &'a ic_agent::Agent);
impl<'a> Service<'a> {
    pub async fn accept_cycles(&self) -> Result<()> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "acceptCycles")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn clear_active_uploads(&self) -> Result<()> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "clearActiveUploads")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn delete_file(&self, arg0: &FileId1) -> Result<()> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "deleteFile")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn download_chunk(&self, arg0: &FileId1, arg1: &candid::Nat) -> Result<Result8> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "downloadChunk")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result8)?)
    }
    pub async fn finish_uploads(&self, arg0: &Vec<FileId1>) -> Result<Result6> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "finishUploads")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result6)?)
    }
    pub async fn get_file_ids_range(
        &self,
        arg0: &candid::Nat,
        arg1: &candid::Nat,
    ) -> Result<Vec<FileId1>> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "getFileIdsRange")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<FileId1>)?)
    }
    pub async fn get_file_meta(&self, arg0: &FileId1) -> Result<Result7> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "getFileMeta")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result7)?)
    }
    pub async fn get_stats(&self) -> Result<StorageStats1> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getStats")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, StorageStats1)?)
    }
    pub async fn start_upload(&self, arg0: &FileMeta) -> Result<Result6> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "startUpload")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result6)?)
    }
    pub async fn update_file_owners(
        &self,
        arg0: &FileId1,
        arg1: &Vec<Principal>,
    ) -> Result<Result6> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "updateFileOwners")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result6)?)
    }
    pub async fn upload_chunk(
        &self,
        arg0: &FileId1,
        arg1: &candid::Nat,
        arg2: &Chunk,
    ) -> Result<Result6> {
        let args = Encode!(&arg0, &arg1, &arg2)?;
        let bytes = self
            .1
            .update(&self.0, "uploadChunk")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result6)?)
    }
}
pub const CANISTER_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 144, 3, 243, 1, 1]); // gl576-4yaaa-aaaam-qapzq-cai
