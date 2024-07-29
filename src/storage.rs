// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
type Result<T> = std::result::Result<T, ic_agent::AgentError>;

pub type FileId1 = String;
pub type Chunk = serde_bytes::ByteBuf;
pub type Err1 = String;
pub type Result8 = candid::MotokoResult<Chunk, Err1>;
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

pub struct Service<'a>(pub Principal, pub &'a ic_agent::Agent);
impl<'a> Service<'a> {
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
}
pub const CANISTER_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 144, 3, 243, 1, 1]); // gl576-4yaaa-aaaam-qapzq-cai
