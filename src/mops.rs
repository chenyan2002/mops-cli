// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
type Result<T> = std::result::Result<T, ic_agent::AgentError>;

pub type PackageName = String;
pub type PackageVersion = String;
pub type FileId = String;
pub type Err = String;
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
pub type Time = candid::Int;
#[derive(CandidType, Deserialize, Debug)]
pub struct PackagePublication {
    pub storage: Principal,
    pub time: Time,
    pub user: Principal,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct PackageSummary1 {
    #[serde(rename = "ownerInfo")]
    pub owner_info: User,
    pub owner: Principal,
    #[serde(rename = "depAlias")]
    pub dep_alias: String,
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
    #[serde(rename = "depAlias")]
    pub dep_alias: String,
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
    #[serde(rename = "depAlias")]
    pub dep_alias: String,
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

pub struct Service<'a>(pub Principal, pub &'a ic_agent::Agent);
impl<'a> Service<'a> {
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
}
pub const CANISTER_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 144, 1, 124, 1, 1]); // oknww-riaaa-aaaam-qaf6a-cai
