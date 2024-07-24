// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
type Result<T> = std::result::Result<T, ic_agent::AgentError>;

pub type PublishingId = String;
pub type Err = String;
pub type Result_ = candid::MotokoResult<(), Err>;
pub type Text = String;
pub type PackageName = String;
pub type PackageVersion = String;
pub type PackageId = String;
pub type Time = candid::Int;
#[derive(CandidType, Deserialize, Debug)]
pub struct DownloadsSnapshot1 {
    #[serde(rename = "startTime")]
    pub start_time: Time,
    #[serde(rename = "endTime")]
    pub end_time: Time,
    pub downloads: candid::Nat,
}
pub type FileId = String;
pub type Result8 = candid::MotokoResult<Vec<(FileId, serde_bytes::ByteBuf)>, Err>;
pub type Result7 = candid::MotokoResult<Vec<FileId>, Err>;
#[derive(CandidType, Deserialize, Debug)]
pub enum SemverPart {
    #[serde(rename = "major")]
    Major,
    #[serde(rename = "minor")]
    Minor,
    #[serde(rename = "patch")]
    Patch,
}
pub type Result6 = candid::MotokoResult<Vec<(PackageName, PackageVersion)>, Err>;
pub type Result5 = candid::MotokoResult<PackageVersion, Err>;
#[derive(CandidType, Deserialize, Debug)]
pub struct User {
    pub id: Principal,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    pub twitter: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub name: String,
    pub site: String,
    pub email: String,
    #[serde(rename = "twitterVerified")]
    pub twitter_verified: bool,
    #[serde(rename = "githubVerified")]
    pub github_verified: bool,
    pub github: String,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum DepsStatus {
    #[serde(rename = "allLatest")]
    AllLatest,
    #[serde(rename = "tooOld")]
    TooOld,
    #[serde(rename = "updatesAvailable")]
    UpdatesAvailable,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageQuality {
    #[serde(rename = "depsStatus")]
    pub deps_status: DepsStatus,
    #[serde(rename = "hasDescription")]
    pub has_description: bool,
    #[serde(rename = "hasKeywords")]
    pub has_keywords: bool,
    #[serde(rename = "hasLicense")]
    pub has_license: bool,
    #[serde(rename = "hasDocumentation")]
    pub has_documentation: bool,
    #[serde(rename = "hasTests")]
    pub has_tests: bool,
    #[serde(rename = "hasRepository")]
    pub has_repository: bool,
    #[serde(rename = "hasReleaseNotes")]
    pub has_release_notes: bool,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct Script {
    pub value: String,
    pub name: String,
}
pub type PackageName1 = String;
#[derive(CandidType, Deserialize, Debug)]
pub struct DependencyV2 {
    pub name: PackageName1,
    pub repo: String,
    pub version: String,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct Requirement {
    pub value: String,
    pub name: String,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageConfigV3 {
    pub dfx: String,
    pub moc: String,
    pub scripts: Vec<Script>,
    #[serde(rename = "baseDir")]
    pub base_dir: String,
    pub documentation: String,
    pub name: PackageName1,
    pub homepage: String,
    pub description: String,
    pub version: String,
    pub keywords: Vec<String>,
    pub donation: String,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Vec<DependencyV2>,
    pub repository: String,
    pub dependencies: Vec<DependencyV2>,
    pub requirements: Vec<Requirement>,
    pub license: String,
    pub readme: String,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct PackagePublication {
    pub storage: Principal,
    pub time: Time,
    pub user: Principal,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageSummary {
    #[serde(rename = "ownerInfo")]
    pub owner_info: User,
    pub owner: Principal,
    pub quality: PackageQuality,
    #[serde(rename = "downloadsTotal")]
    pub downloads_total: candid::Nat,
    #[serde(rename = "downloadsInLast30Days")]
    pub downloads_in_last_30_days: candid::Nat,
    #[serde(rename = "downloadsInLast7Days")]
    pub downloads_in_last_7_days: candid::Nat,
    pub config: PackageConfigV3,
    pub publication: PackagePublication,
}
pub type BenchmarkMetric = String;
#[derive(CandidType, Deserialize, Debug)]
pub struct Benchmark {
    pub gc: String,
    pub metrics: Vec<(BenchmarkMetric, Vec<Vec<candid::Int>>)>,
    pub cols: Vec<String>,
    pub file: String,
    pub name: String,
    pub rows: Vec<String>,
    pub description: String,
    #[serde(rename = "compilerVersion")]
    pub compiler_version: String,
    pub compiler: String,
    pub replica: String,
    #[serde(rename = "replicaVersion")]
    pub replica_version: String,
    #[serde(rename = "forceGC")]
    pub force_gc: bool,
}
pub type Benchmarks1 = Vec<Benchmark>;
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageSummary1 {
    #[serde(rename = "ownerInfo")]
    pub owner_info: User,
    pub owner: Principal,
    pub quality: PackageQuality,
    #[serde(rename = "downloadsTotal")]
    pub downloads_total: candid::Nat,
    #[serde(rename = "downloadsInLast30Days")]
    pub downloads_in_last_30_days: candid::Nat,
    #[serde(rename = "downloadsInLast7Days")]
    pub downloads_in_last_7_days: candid::Nat,
    pub config: PackageConfigV3,
    pub publication: PackagePublication,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct TestStats1 {
    #[serde(rename = "passedNames")]
    pub passed_names: Vec<String>,
    pub passed: candid::Nat,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct DownloadsSnapshot {
    #[serde(rename = "startTime")]
    pub start_time: Time,
    #[serde(rename = "endTime")]
    pub end_time: Time,
    pub downloads: candid::Nat,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageFileStatsPublic {
    #[serde(rename = "sourceFiles")]
    pub source_files: candid::Nat,
    #[serde(rename = "sourceSize")]
    pub source_size: candid::Nat,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct TestsChanges {
    #[serde(rename = "addedNames")]
    pub added_names: Vec<String>,
    #[serde(rename = "removedNames")]
    pub removed_names: Vec<String>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct DepChange {
    #[serde(rename = "oldVersion")]
    pub old_version: String,
    pub name: String,
    #[serde(rename = "newVersion")]
    pub new_version: String,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageChanges {
    pub tests: TestsChanges,
    pub deps: Vec<DepChange>,
    #[serde(rename = "curBenchmarks")]
    pub cur_benchmarks: Benchmarks1,
    #[serde(rename = "prevBenchmarks")]
    pub prev_benchmarks: Benchmarks1,
    pub notes: String,
    #[serde(rename = "devDeps")]
    pub dev_deps: Vec<DepChange>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageSummaryWithChanges1 {
    #[serde(rename = "ownerInfo")]
    pub owner_info: User,
    pub owner: Principal,
    pub quality: PackageQuality,
    #[serde(rename = "downloadsTotal")]
    pub downloads_total: candid::Nat,
    #[serde(rename = "downloadsInLast30Days")]
    pub downloads_in_last_30_days: candid::Nat,
    #[serde(rename = "downloadsInLast7Days")]
    pub downloads_in_last_7_days: candid::Nat,
    pub config: PackageConfigV3,
    pub changes: PackageChanges,
    pub publication: PackagePublication,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageDetails {
    pub benchmarks: Benchmarks1,
    #[serde(rename = "ownerInfo")]
    pub owner_info: User,
    pub owner: Principal,
    pub deps: Vec<PackageSummary1>,
    pub quality: PackageQuality,
    #[serde(rename = "testStats")]
    pub test_stats: TestStats1,
    #[serde(rename = "downloadsTotal")]
    pub downloads_total: candid::Nat,
    #[serde(rename = "downloadsInLast30Days")]
    pub downloads_in_last_30_days: candid::Nat,
    #[serde(rename = "downloadTrend")]
    pub download_trend: Vec<DownloadsSnapshot>,
    #[serde(rename = "fileStats")]
    pub file_stats: PackageFileStatsPublic,
    #[serde(rename = "versionHistory")]
    pub version_history: Vec<PackageSummaryWithChanges1>,
    pub dependents: Vec<PackageSummary1>,
    #[serde(rename = "devDeps")]
    pub dev_deps: Vec<PackageSummary1>,
    #[serde(rename = "downloadsInLast7Days")]
    pub downloads_in_last_7_days: candid::Nat,
    pub config: PackageConfigV3,
    pub changes: PackageChanges,
    pub publication: PackagePublication,
}
pub type Result4 = candid::MotokoResult<PackageDetails, Err>;
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageSummaryWithChanges {
    #[serde(rename = "ownerInfo")]
    pub owner_info: User,
    pub owner: Principal,
    pub quality: PackageQuality,
    #[serde(rename = "downloadsTotal")]
    pub downloads_total: candid::Nat,
    #[serde(rename = "downloadsInLast30Days")]
    pub downloads_in_last_30_days: candid::Nat,
    #[serde(rename = "downloadsInLast7Days")]
    pub downloads_in_last_7_days: candid::Nat,
    pub config: PackageConfigV3,
    pub changes: PackageChanges,
    pub publication: PackagePublication,
}
pub type StorageId = Principal;
#[derive(CandidType, Deserialize, Debug)]
pub struct StorageStats {
    #[serde(rename = "fileCount")]
    pub file_count: candid::Nat,
    #[serde(rename = "cyclesBalance")]
    pub cycles_balance: candid::Nat,
    #[serde(rename = "memorySize")]
    pub memory_size: candid::Nat,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct User1 {
    pub id: Principal,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    pub twitter: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub name: String,
    pub site: String,
    pub email: String,
    #[serde(rename = "twitterVerified")]
    pub twitter_verified: bool,
    #[serde(rename = "githubVerified")]
    pub github_verified: bool,
    pub github: String,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct Header(pub String, pub String);
#[derive(CandidType, Deserialize, Debug)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub body: serde_bytes::ByteBuf,
    pub headers: Vec<Header>,
    pub certificate_version: Option<u16>,
}
pub type StreamingToken = serde_bytes::ByteBuf;
#[derive(CandidType, Deserialize, Debug)]
pub struct StreamingCallbackResponse {
    pub token: Option<StreamingToken>,
    pub body: serde_bytes::ByteBuf,
}
candid::define_function!(pub StreamingCallback : (StreamingToken) -> (
    Option<StreamingCallbackResponse>,
  ) query);
#[derive(CandidType, Deserialize, Debug)]
pub enum StreamingStrategy {
    Callback {
        token: StreamingToken,
        callback: StreamingCallback,
    },
}
#[derive(CandidType, Deserialize, Debug)]
pub struct Response {
    pub body: serde_bytes::ByteBuf,
    pub headers: Vec<Header>,
    pub upgrade: Option<bool>,
    pub streaming_strategy: Option<StreamingStrategy>,
    pub status_code: u16,
}
pub type PageCount = candid::Nat;
pub type Result1 = candid::MotokoResult<(), String>;
pub type Result3 = candid::MotokoResult<FileId, Err>;
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageConfigV3Publishing {
    pub dfx: String,
    pub moc: String,
    pub scripts: Vec<Script>,
    #[serde(rename = "baseDir")]
    pub base_dir: String,
    pub documentation: String,
    pub name: PackageName1,
    pub homepage: String,
    pub description: String,
    pub version: String,
    pub keywords: Vec<String>,
    pub donation: String,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Vec<DependencyV2>,
    pub repository: String,
    pub dependencies: Vec<DependencyV2>,
    pub requirements: Option<Vec<Requirement>>,
    pub license: String,
    pub readme: String,
}
pub type Result2 = candid::MotokoResult<PublishingId, Err>;
#[derive(CandidType, Deserialize, Debug)]
pub struct HttpHeader {
    pub value: String,
    pub name: String,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct HttpResponse {
    pub status: candid::Nat,
    pub body: serde_bytes::ByteBuf,
    pub headers: Vec<HttpHeader>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct HttpTransformArg {
    pub context: serde_bytes::ByteBuf,
    pub response: HttpResponse,
}
pub type Benchmarks = Vec<Benchmark>;
#[derive(CandidType, Deserialize, Debug)]
pub struct TestStats {
    #[serde(rename = "passedNames")]
    pub passed_names: Vec<String>,
    pub passed: candid::Nat,
}

pub struct Service<'a>(pub Principal, pub &'a ic_agent::Agent);
impl<'a> Service<'a> {
    pub async fn backup(&self) -> Result<()> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "backup")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn compute_hashes_for_existing_files(&self) -> Result<()> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "computeHashesForExistingFiles")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn finish_publish(&self, arg0: &PublishingId) -> Result<Result_> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "finishPublish")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result_)?)
    }
    pub async fn get_api_version(&self) -> Result<Text> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getApiVersion")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Text)?)
    }
    pub async fn get_backup_canister_id(&self) -> Result<Principal> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getBackupCanisterId")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Principal)?)
    }
    pub async fn get_default_packages(
        &self,
        arg0: &String,
    ) -> Result<Vec<(PackageName, PackageVersion)>> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "getDefaultPackages")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<(PackageName, PackageVersion,)>)?)
    }
    pub async fn get_download_trend_by_package_id(
        &self,
        arg0: &PackageId,
    ) -> Result<Vec<DownloadsSnapshot1>> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "getDownloadTrendByPackageId")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<DownloadsSnapshot1>)?)
    }
    pub async fn get_download_trend_by_package_name(
        &self,
        arg0: &PackageName,
    ) -> Result<Vec<DownloadsSnapshot1>> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "getDownloadTrendByPackageName")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<DownloadsSnapshot1>)?)
    }
    pub async fn get_file_hashes(
        &self,
        arg0: &PackageName,
        arg1: &PackageVersion,
    ) -> Result<Result8> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "getFileHashes")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result8)?)
    }
    pub async fn get_file_hashes_by_package_ids(
        &self,
        arg0: &Vec<PackageId>,
    ) -> Result<Vec<(PackageId, Vec<(FileId, serde_bytes::ByteBuf)>)>> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "getFileHashesByPackageIds")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(
            &bytes,
            Vec<(PackageId, Vec<(FileId, serde_bytes::ByteBuf,)>,)>
        )?)
    }
    pub async fn get_file_hashes_query(
        &self,
        arg0: &PackageName,
        arg1: &PackageVersion,
    ) -> Result<Result8> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "getFileHashesQuery")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result8)?)
    }
    pub async fn get_file_ids(&self, arg0: &PackageName, arg1: &PackageVersion) -> Result<Result7> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "getFileIds")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result7)?)
    }
    pub async fn get_highest_semver_batch(
        &self,
        arg0: &Vec<(PackageName, PackageVersion, SemverPart)>,
    ) -> Result<Result6> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "getHighestSemverBatch")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result6)?)
    }
    pub async fn get_highest_version(&self, arg0: &PackageName) -> Result<Result5> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "getHighestVersion")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result5)?)
    }
    pub async fn get_most_downloaded_packages(&self) -> Result<Vec<PackageSummary>> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getMostDownloadedPackages")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<PackageSummary>)?)
    }
    pub async fn get_most_downloaded_packages_in_7_days(&self) -> Result<Vec<PackageSummary>> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getMostDownloadedPackagesIn7Days")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<PackageSummary>)?)
    }
    pub async fn get_new_packages(&self) -> Result<Vec<PackageSummary>> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getNewPackages")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<PackageSummary>)?)
    }
    pub async fn get_package_details(
        &self,
        arg0: &PackageName,
        arg1: &PackageVersion,
    ) -> Result<Result4> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "getPackageDetails")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result4)?)
    }
    pub async fn get_packages_by_category(&self) -> Result<Vec<(String, Vec<PackageSummary>)>> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getPackagesByCategory")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<(String, Vec<PackageSummary>,)>)?)
    }
    pub async fn get_recently_updated_packages(&self) -> Result<Vec<PackageSummaryWithChanges>> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getRecentlyUpdatedPackages")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<PackageSummaryWithChanges>)?)
    }
    pub async fn get_storages_stats(&self) -> Result<Vec<(StorageId, StorageStats)>> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getStoragesStats")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<(StorageId, StorageStats,)>)?)
    }
    pub async fn get_total_downloads(&self) -> Result<candid::Nat> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getTotalDownloads")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, candid::Nat)?)
    }
    pub async fn get_total_packages(&self) -> Result<candid::Nat> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "getTotalPackages")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, candid::Nat)?)
    }
    pub async fn get_user(&self, arg0: &Principal) -> Result<Option<User1>> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "getUser")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Option<User1>)?)
    }
    pub async fn http_request(&self, arg0: &Request) -> Result<Response> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "http_request")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Response)?)
    }
    pub async fn notify_install(&self, arg0: &PackageName, arg1: &PackageVersion) -> Result<()> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "notifyInstall")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn notify_installs(&self, arg0: &Vec<(PackageName, PackageVersion)>) -> Result<()> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "notifyInstalls")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn restore(&self, arg0: &candid::Nat) -> Result<()> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "restore")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn search(
        &self,
        arg0: &Text,
        arg1: &Option<candid::Nat>,
        arg2: &Option<candid::Nat>,
    ) -> Result<(Vec<PackageSummary>, PageCount)> {
        let args = Encode!(&arg0, &arg1, &arg2)?;
        let bytes = self
            .1
            .query(&self.0, "search")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<PackageSummary>, PageCount)?)
    }
    pub async fn set_user_prop(&self, arg0: &String, arg1: &String) -> Result<Result1> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "setUserProp")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result1)?)
    }
    pub async fn start_file_upload(
        &self,
        arg0: &PublishingId,
        arg1: &Text,
        arg2: &candid::Nat,
        arg3: &serde_bytes::ByteBuf,
    ) -> Result<Result3> {
        let args = Encode!(&arg0, &arg1, &arg2, &arg3)?;
        let bytes = self
            .1
            .update(&self.0, "startFileUpload")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result3)?)
    }
    pub async fn start_publish(&self, arg0: &PackageConfigV3Publishing) -> Result<Result2> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "startPublish")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result2)?)
    }
    pub async fn transfer_ownership(
        &self,
        arg0: &PackageName,
        arg1: &Principal,
    ) -> Result<Result1> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "transferOwnership")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result1)?)
    }
    pub async fn transform_request(&self, arg0: &HttpTransformArg) -> Result<HttpResponse> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "transformRequest")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, HttpResponse)?)
    }
    pub async fn upload_benchmarks(
        &self,
        arg0: &PublishingId,
        arg1: &Benchmarks,
    ) -> Result<Result_> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "uploadBenchmarks")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result_)?)
    }
    pub async fn upload_file_chunk(
        &self,
        arg0: &PublishingId,
        arg1: &FileId,
        arg2: &candid::Nat,
        arg3: &serde_bytes::ByteBuf,
    ) -> Result<Result_> {
        let args = Encode!(&arg0, &arg1, &arg2, &arg3)?;
        let bytes = self
            .1
            .update(&self.0, "uploadFileChunk")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result_)?)
    }
    pub async fn upload_notes(&self, arg0: &PublishingId, arg1: &String) -> Result<Result_> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "uploadNotes")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result_)?)
    }
    pub async fn upload_test_stats(
        &self,
        arg0: &PublishingId,
        arg1: &TestStats,
    ) -> Result<Result_> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "uploadTestStats")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result_)?)
    }
}
pub const CANISTER_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 144, 1, 124, 1, 1]); // oknww-riaaa-aaaam-qaf6a-cai
