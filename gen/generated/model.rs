use endpoint_libs::libs::error_code::ErrorCode;
use endpoint_libs::libs::types::*;
use endpoint_libs::libs::ws::*;
use num_derive::FromPrimitive;
use serde::*;
use strum_macros::{Display, EnumString};
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    FromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    Hash,
)]
pub enum EnumFundStatus {
    /// Fund awaiting approval
    Pending = 1,
    /// Fund open for investment
    Active = 2,
    /// Fund is no longer active
    Closed = 3,
    /// Fund is temporarily suspended
    Suspended = 4,
}
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    FromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    Hash,
)]
pub enum EnumFundStrategy {
    /// Staking strategy
    Staking = 1,
    /// Custody strategy
    Custody = 2,
    /// Investing strategy
    Investing = 3,
}
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    FromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    Hash,
)]
pub enum EnumRole {
    /// The user role defines a person interacting with the service
    User = 1,
    /// The fund manager role defines the admin of the fund, which provides services for the users
    FundManager = 2,
    /// The admin role defines the service admin, controlling both users and fund managers
    Admin = 3,
}
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    FromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    Hash,
)]
pub enum EnumService {
    ///
    Auth = 1,
    ///
    User = 2,
}
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    FromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    Hash,
)]
pub enum EnumServicesOfInterest {
    /// Custody services
    Custody = 1,
    /// Staking services
    Staking = 2,
    /// Investing services
    Investing = 3,
}
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    FromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    Hash,
)]
pub enum EnumWalletStatus {
    /// Active wallet
    Active = 1,
    /// Inactive wallet
    Inactive = 2,
}
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    FromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    Hash,
)]
pub enum EnumWalletType {
    /// External wallet type
    External = 1,
    /// Internal wallet type
    Internal = 2,
}
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    FromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    Hash,
)]
pub enum EnumEndpoint {
    ///
    Login = 10020,
    ///
    SignUp = 10021,
    ///
    UserPing = 40000,
    ///
    UserGetTopFunds = 41000,
    ///
    UserGetFundDetails = 41010,
    ///
    UserListFunds = 41020,
    ///
    FundManagerSignup = 41030,
    ///
    FundManagerUpdateLiquidity = 41040,
    ///
    AdminListPendingFunds = 42010,
    ///
    AdminListUsers = 42020,
    ///
    AdminApproveRejectFund = 42030,
    ///
    AdminApproveLiquidityUpdate = 42040,
    ///
    AdminListFundsWithPendingLiquidity = 43000,
    ///
    CaptureLead = 43010,
    ///
    AdminGetLeads = 43020,
    ///
    UserConnectWallet = 43030,
    ///
    UserWallet = 43040,
    ///
    FundManagerManageAllocations = 43050,
    ///
    FundManagerAdjustStrategy = 43060,
    ///
    FundManagerPerformanceReport = 43070,
    ///
    UserGetYieldData = 43080,
    ///
    AddYieldData = 43090,
}

impl EnumEndpoint {
    pub fn schema(&self) -> endpoint_libs::model::EndpointSchema {
        let schema = match self {
            Self::Login => LoginRequest::SCHEMA,
            Self::SignUp => SignUpRequest::SCHEMA,
            Self::UserPing => UserPingRequest::SCHEMA,
            Self::UserGetTopFunds => UserGetTopFundsRequest::SCHEMA,
            Self::UserGetFundDetails => UserGetFundDetailsRequest::SCHEMA,
            Self::UserListFunds => UserListFundsRequest::SCHEMA,
            Self::FundManagerSignup => FundManagerSignupRequest::SCHEMA,
            Self::FundManagerUpdateLiquidity => FundManagerUpdateLiquidityRequest::SCHEMA,
            Self::AdminListPendingFunds => AdminListPendingFundsRequest::SCHEMA,
            Self::AdminListUsers => AdminListUsersRequest::SCHEMA,
            Self::AdminApproveRejectFund => AdminApproveRejectFundRequest::SCHEMA,
            Self::AdminApproveLiquidityUpdate => AdminApproveLiquidityUpdateRequest::SCHEMA,
            Self::AdminListFundsWithPendingLiquidity => {
                AdminListFundsWithPendingLiquidityRequest::SCHEMA
            }
            Self::CaptureLead => CaptureLeadRequest::SCHEMA,
            Self::AdminGetLeads => AdminGetLeadsRequest::SCHEMA,
            Self::UserConnectWallet => UserConnectWalletRequest::SCHEMA,
            Self::UserWallet => UserWalletRequest::SCHEMA,
            Self::FundManagerManageAllocations => FundManagerManageAllocationsRequest::SCHEMA,
            Self::FundManagerAdjustStrategy => FundManagerAdjustStrategyRequest::SCHEMA,
            Self::FundManagerPerformanceReport => FundManagerPerformanceReportRequest::SCHEMA,
            Self::UserGetYieldData => UserGetYieldDataRequest::SCHEMA,
            Self::AddYieldData => AddYieldDataRequest::SCHEMA,
        };
        serde_json::from_str(schema).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ErrorXxx {}
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    FromPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    Hash,
)]
pub enum EnumErrorCode {
    /// None Please populate error_codes.json
    Xxx = 0,
}

impl From<EnumErrorCode> for ErrorCode {
    fn from(e: EnumErrorCode) -> Self {
        ErrorCode::new(e as _)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddYieldDataRequest {
    pub cagr: f64,
    pub user_id: i64,
    pub wallet_id: i64,
    pub yield_amount: f64,
    pub yield_date: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddYieldDataResponse {
    pub yield_id: i64,
    pub message: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminApproveLiquidityUpdateRequest {
    pub fund_name: String,
    pub approve: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminApproveLiquidityUpdateResponse {
    pub fund_name: String,
    pub old_liquidity: f64,
    pub new_liquidity: f64,
    pub approved: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminApproveRejectFundRequest {
    pub comment: String,
    pub fund_name: String,
    pub new_status: EnumFundStatus,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminApproveRejectFundResponse {
    pub fund_name: String,
    pub new_status: EnumFundStatus,
    pub comment: String,
    pub legal_documents: Vec<LegalDocument>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminGetLeadsRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminGetLeadsResponse {
    pub data: Vec<LeadModel>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminListFundsWithPendingLiquidityRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminListFundsWithPendingLiquidityResponse {
    pub data: Vec<FundWithPendingLiquidity>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminListPendingFundsRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminListPendingFundsResponse {
    pub data: Vec<PendingFund>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminListUsersRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminListUsersResponse {
    pub data: Vec<UserModel>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CaptureLeadRequest {
    pub country: String,
    pub email: String,
    pub message: String,
    pub name: String,
    pub phone: String,
    pub services_of_interest: EnumServicesOfInterest,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CaptureLeadResponse {
    pub lead_id: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Fund {
    pub fund_name: String,
    pub fund_start_date: i64,
    pub fund_target_apy: f64,
    pub fund_address: String,
    pub company_name: String,
    pub email_address: String,
    pub phone_number: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerAdjustStrategyRequest {
    pub fund_name: String,
    pub new_strategy: EnumFundStrategy,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerAdjustStrategyResponse {
    pub new_strategy: EnumFundStrategy,
    pub fund_name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerManageAllocationsRequest {
    pub allocation_data: String,
    pub fund_name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerManageAllocationsResponse {
    pub message: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerPerformanceReportRequest {
    pub from_timestamp: i64,
    pub fund_name: String,
    pub to_timestamp: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerPerformanceReportResponse {
    pub report_url: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerSignupRequest {
    pub allocation_data: String,
    pub aum: f64,
    pub company_name: String,
    pub country: String,
    pub description: String,
    pub fund_address: String,
    pub fund_name: String,
    pub fund_start_date: i64,
    pub fund_strategy: EnumFundStrategy,
    pub fund_target_apy: f64,
    pub last_updated_at: i64,
    pub legal_documents: Vec<LegalDocument>,
    pub manager_company_name: String,
    pub manager_phone_number: String,
    pub manager_email: String,
    pub strategy_description: String,
    pub performance_reports: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerSignupResponse {
    pub fund_id: i64,
    pub message: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerUpdateLiquidityRequest {
    pub fund_name: String,
    pub new_liquidity: f64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundManagerUpdateLiquidityResponse {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundWithPendingLiquidity {
    pub fund_name: String,
    pub pending_liquidity: f64,
    pub fund_address: String,
    pub target_apy: f64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LeadModel {
    pub lead_id: i64,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub country: String,
    pub services_of_interest: EnumServicesOfInterest,
    pub message: String,
    pub created_at: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LegalDocument {
    pub filename: String,
    pub base64_data: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub username: String,
    pub display_name: String,
    #[serde(default)]
    pub avatar: Option<String>,
    pub role: EnumRole,
    pub user_id: i64,
    pub user_token: uuid::Uuid,
    pub admin_token: uuid::Uuid,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PendingFund {
    pub fund_name: String,
    pub fund_start_date: i64,
    pub fund_target_apy: f64,
    pub fund_address: String,
    pub company_name: String,
    pub email_address: String,
    pub phone_number: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignUpRequest {
    pub password: String,
    pub username: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignUpResponse {
    pub username: String,
    pub display_name: String,
    #[serde(default)]
    pub avatar: Option<String>,
    pub role: EnumRole,
    pub user_id: i64,
    pub user_token: uuid::Uuid,
    pub admin_token: uuid::Uuid,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopFund {
    pub fund_name: String,
    pub fund_liquidity: f64,
    pub fund_target_apy: f64,
    pub fund_address: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserConnectWalletRequest {
    pub user_id: i64,
    pub wallet_address: String,
    pub wallet_type: EnumWalletType,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserConnectWalletResponse {
    pub wallet_id: i64,
    pub message: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserGetFundDetailsRequest {
    pub fund_name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserGetFundDetailsResponse {
    pub fund_name: String,
    pub fund_liquidity: f64,
    pub fund_target_apy: f64,
    pub fund_address: String,
    pub fund_start_date: i64,
    pub description: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserGetTopFundsRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserGetTopFundsResponse {
    pub data: Vec<TopFund>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserGetYieldDataRequest {
    pub from_timestamp: i64,
    pub to_timestamp: i64,
    pub user_id: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserGetYieldDataResponse {
    pub total_yield: f64,
    pub daily_yield: f64,
    pub annual_yield: f64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserListFundsRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserListFundsResponse {
    pub data: Vec<Fund>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserModel {
    pub user_id: i64,
    pub name: String,
    pub email: String,
    pub role: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserPingRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserPingResponse {
    pub pong: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserWalletInfo {
    pub wallet_id: i64,
    pub wallet_address: String,
    pub wallet_type: EnumWalletType,
    pub balance: f64,
    pub status: EnumWalletStatus,
    pub created_at: i64,
    pub updated_at: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserWalletRequest {
    pub user_id: i64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserWalletResponse {
    pub wallets: Vec<UserWalletInfo>,
}
impl WsRequest for LoginRequest {
    type Response = LoginResponse;
    const METHOD_ID: u32 = 10020;
    const SCHEMA: &'static str = r#"{
  "name": "Login",
  "code": 10020,
  "parameters": [
    {
      "name": "username",
      "ty": "String"
    },
    {
      "name": "password",
      "ty": "String"
    }
  ],
  "returns": [
    {
      "name": "username",
      "ty": "String"
    },
    {
      "name": "display_name",
      "ty": "String"
    },
    {
      "name": "avatar",
      "ty": {
        "Optional": "String"
      }
    },
    {
      "name": "role",
      "ty": {
        "EnumRef": "role"
      }
    },
    {
      "name": "user_id",
      "ty": "BigInt"
    },
    {
      "name": "user_token",
      "ty": "UUID"
    },
    {
      "name": "admin_token",
      "ty": "UUID"
    }
  ],
  "stream_response": null,
  "description": "User login",
  "json_schema": null
}"#;
}
impl WsResponse for LoginResponse {
    type Request = LoginRequest;
}

impl WsRequest for SignUpRequest {
    type Response = SignUpResponse;
    const METHOD_ID: u32 = 10021;
    const SCHEMA: &'static str = r#"{
  "name": "SignUp",
  "code": 10021,
  "parameters": [
    {
      "name": "password",
      "ty": "String"
    },
    {
      "name": "username",
      "ty": "String"
    }
  ],
  "returns": [
    {
      "name": "username",
      "ty": "String"
    },
    {
      "name": "display_name",
      "ty": "String"
    },
    {
      "name": "avatar",
      "ty": {
        "Optional": "String"
      }
    },
    {
      "name": "role",
      "ty": {
        "EnumRef": "role"
      }
    },
    {
      "name": "user_id",
      "ty": "BigInt"
    },
    {
      "name": "user_token",
      "ty": "UUID"
    },
    {
      "name": "admin_token",
      "ty": "UUID"
    }
  ],
  "stream_response": null,
  "description": "User registration",
  "json_schema": null
}"#;
}
impl WsResponse for SignUpResponse {
    type Request = SignUpRequest;
}

impl WsRequest for UserPingRequest {
    type Response = UserPingResponse;
    const METHOD_ID: u32 = 40000;
    const SCHEMA: &'static str = r#"{
  "name": "UserPing",
  "code": 40000,
  "parameters": [],
  "returns": [
    {
      "name": "pong",
      "ty": "String"
    }
  ],
  "stream_response": null,
  "description": "A simple health check: receives 'ping' and returns 'pong'.",
  "json_schema": null
}"#;
}
impl WsResponse for UserPingResponse {
    type Request = UserPingRequest;
}

impl WsRequest for UserGetTopFundsRequest {
    type Response = UserGetTopFundsResponse;
    const METHOD_ID: u32 = 41000;
    const SCHEMA: &'static str = r#"{
  "name": "UserGetTopFunds",
  "code": 41000,
  "parameters": [],
  "returns": [
    {
      "name": "data",
      "ty": {
        "DataTable": {
          "name": "TopFund",
          "fields": [
            {
              "name": "fund_name",
              "ty": "String"
            },
            {
              "name": "fund_liquidity",
              "ty": "Numeric"
            },
            {
              "name": "fund_target_apy",
              "ty": "Numeric"
            },
            {
              "name": "fund_address",
              "ty": "String"
            }
          ]
        }
      }
    }
  ],
  "stream_response": null,
  "description": "Fetch the top 3 funds by liquidity or other ranking metric.",
  "json_schema": null
}"#;
}
impl WsResponse for UserGetTopFundsResponse {
    type Request = UserGetTopFundsRequest;
}

impl WsRequest for UserGetFundDetailsRequest {
    type Response = UserGetFundDetailsResponse;
    const METHOD_ID: u32 = 41010;
    const SCHEMA: &'static str = r#"{
  "name": "UserGetFundDetails",
  "code": 41010,
  "parameters": [
    {
      "name": "fund_name",
      "ty": "String"
    }
  ],
  "returns": [
    {
      "name": "fund_name",
      "ty": "String"
    },
    {
      "name": "fund_liquidity",
      "ty": "Numeric"
    },
    {
      "name": "fund_target_apy",
      "ty": "Numeric"
    },
    {
      "name": "fund_address",
      "ty": "String"
    },
    {
      "name": "fund_start_date",
      "ty": "BigInt"
    },
    {
      "name": "description",
      "ty": "String"
    }
  ],
  "stream_response": null,
  "description": "Return details for the specified fund.",
  "json_schema": null
}"#;
}
impl WsResponse for UserGetFundDetailsResponse {
    type Request = UserGetFundDetailsRequest;
}

impl WsRequest for UserListFundsRequest {
    type Response = UserListFundsResponse;
    const METHOD_ID: u32 = 41020;
    const SCHEMA: &'static str = r#"{
  "name": "UserListFunds",
  "code": 41020,
  "parameters": [],
  "returns": [
    {
      "name": "data",
      "ty": {
        "DataTable": {
          "name": "Fund",
          "fields": [
            {
              "name": "fund_name",
              "ty": "String"
            },
            {
              "name": "fund_start_date",
              "ty": "BigInt"
            },
            {
              "name": "fund_target_apy",
              "ty": "Numeric"
            },
            {
              "name": "fund_address",
              "ty": "String"
            },
            {
              "name": "company_name",
              "ty": "String"
            },
            {
              "name": "email_address",
              "ty": "String"
            },
            {
              "name": "phone_number",
              "ty": "String"
            }
          ]
        }
      }
    }
  ],
  "stream_response": null,
  "description": "List all active funds.",
  "json_schema": null
}"#;
}
impl WsResponse for UserListFundsResponse {
    type Request = UserListFundsRequest;
}

impl WsRequest for FundManagerSignupRequest {
    type Response = FundManagerSignupResponse;
    const METHOD_ID: u32 = 41030;
    const SCHEMA: &'static str = r#"{
  "name": "FundManagerSignup",
  "code": 41030,
  "parameters": [
    {
      "name": "allocation_data",
      "ty": "String"
    },
    {
      "name": "aum",
      "ty": "Numeric"
    },
    {
      "name": "company_name",
      "ty": "String"
    },
    {
      "name": "country",
      "ty": "String"
    },
    {
      "name": "description",
      "ty": "String"
    },
    {
      "name": "fund_address",
      "ty": "String"
    },
    {
      "name": "fund_name",
      "ty": "String"
    },
    {
      "name": "fund_start_date",
      "ty": "BigInt"
    },
    {
      "name": "fund_strategy",
      "ty": {
        "EnumRef": "fund_strategy"
      }
    },
    {
      "name": "fund_target_apy",
      "ty": "Numeric"
    },
    {
      "name": "last_updated_at",
      "ty": "BigInt"
    },
    {
      "name": "legal_documents",
      "ty": {
        "DataTable": {
          "name": "LegalDocument",
          "fields": [
            {
              "name": "filename",
              "ty": "String"
            },
            {
              "name": "base64_data",
              "ty": "String"
            }
          ]
        }
      }
    },
    {
      "name": "manager_company_name",
      "ty": "String"
    },
    {
      "name": "manager_phone_number",
      "ty": "String"
    },
    {
      "name": "manager_email",
      "ty": "String"
    },
    {
      "name": "strategy_description",
      "ty": "String"
    },
    {
      "name": "performance_reports",
      "ty": "String"
    }
  ],
  "returns": [
    {
      "name": "fund_id",
      "ty": "BigInt"
    },
    {
      "name": "message",
      "ty": "String"
    }
  ],
  "stream_response": null,
  "description": "Allow a user to register a new fund.",
  "json_schema": null
}"#;
}
impl WsResponse for FundManagerSignupResponse {
    type Request = FundManagerSignupRequest;
}

impl WsRequest for FundManagerUpdateLiquidityRequest {
    type Response = FundManagerUpdateLiquidityResponse;
    const METHOD_ID: u32 = 41040;
    const SCHEMA: &'static str = r#"{
  "name": "FundManagerUpdateLiquidity",
  "code": 41040,
  "parameters": [
    {
      "name": "fund_name",
      "ty": "String"
    },
    {
      "name": "new_liquidity",
      "ty": "Numeric"
    }
  ],
  "returns": [],
  "stream_response": null,
  "description": "Update the specified fund's liquidity (f64).",
  "json_schema": null
}"#;
}
impl WsResponse for FundManagerUpdateLiquidityResponse {
    type Request = FundManagerUpdateLiquidityRequest;
}

impl WsRequest for AdminListPendingFundsRequest {
    type Response = AdminListPendingFundsResponse;
    const METHOD_ID: u32 = 42010;
    const SCHEMA: &'static str = r#"{
  "name": "AdminListPendingFunds",
  "code": 42010,
  "parameters": [],
  "returns": [
    {
      "name": "data",
      "ty": {
        "DataTable": {
          "name": "PendingFund",
          "fields": [
            {
              "name": "fund_name",
              "ty": "String"
            },
            {
              "name": "fund_start_date",
              "ty": "BigInt"
            },
            {
              "name": "fund_target_apy",
              "ty": "Numeric"
            },
            {
              "name": "fund_address",
              "ty": "String"
            },
            {
              "name": "company_name",
              "ty": "String"
            },
            {
              "name": "email_address",
              "ty": "String"
            },
            {
              "name": "phone_number",
              "ty": "String"
            }
          ]
        }
      }
    }
  ],
  "stream_response": null,
  "description": "List funds with pending status (admin role).",
  "json_schema": null
}"#;
}
impl WsResponse for AdminListPendingFundsResponse {
    type Request = AdminListPendingFundsRequest;
}

impl WsRequest for AdminListUsersRequest {
    type Response = AdminListUsersResponse;
    const METHOD_ID: u32 = 42020;
    const SCHEMA: &'static str = r#"{
  "name": "AdminListUsers",
  "code": 42020,
  "parameters": [],
  "returns": [
    {
      "name": "data",
      "ty": {
        "DataTable": {
          "name": "UserModel",
          "fields": [
            {
              "name": "user_id",
              "ty": "BigInt"
            },
            {
              "name": "name",
              "ty": "String"
            },
            {
              "name": "email",
              "ty": "String"
            },
            {
              "name": "role",
              "ty": "String"
            }
          ]
        }
      }
    }
  ],
  "stream_response": null,
  "description": "List all users, including name, email, and role (admin role).",
  "json_schema": null
}"#;
}
impl WsResponse for AdminListUsersResponse {
    type Request = AdminListUsersRequest;
}

impl WsRequest for AdminApproveRejectFundRequest {
    type Response = AdminApproveRejectFundResponse;
    const METHOD_ID: u32 = 42030;
    const SCHEMA: &'static str = r#"{
  "name": "AdminApproveRejectFund",
  "code": 42030,
  "parameters": [
    {
      "name": "comment",
      "ty": "String"
    },
    {
      "name": "fund_name",
      "ty": "String"
    },
    {
      "name": "new_status",
      "ty": {
        "EnumRef": "fund_status"
      }
    }
  ],
  "returns": [
    {
      "name": "fund_name",
      "ty": "String"
    },
    {
      "name": "new_status",
      "ty": {
        "EnumRef": "fund_status"
      }
    },
    {
      "name": "comment",
      "ty": "String"
    },
    {
      "name": "legal_documents",
      "ty": {
        "DataTable": {
          "name": "LegalDocument",
          "fields": [
            {
              "name": "filename",
              "ty": "String"
            },
            {
              "name": "base64_data",
              "ty": "String"
            }
          ]
        }
      }
    }
  ],
  "stream_response": null,
  "description": "Approve or reject a specified fund (admin role).",
  "json_schema": null
}"#;
}
impl WsResponse for AdminApproveRejectFundResponse {
    type Request = AdminApproveRejectFundRequest;
}

impl WsRequest for AdminApproveLiquidityUpdateRequest {
    type Response = AdminApproveLiquidityUpdateResponse;
    const METHOD_ID: u32 = 42040;
    const SCHEMA: &'static str = r#"{
  "name": "AdminApproveLiquidityUpdate",
  "code": 42040,
  "parameters": [
    {
      "name": "fund_name",
      "ty": "String"
    },
    {
      "name": "approve",
      "ty": "Boolean"
    }
  ],
  "returns": [
    {
      "name": "fund_name",
      "ty": "String"
    },
    {
      "name": "old_liquidity",
      "ty": "Numeric"
    },
    {
      "name": "new_liquidity",
      "ty": "Numeric"
    },
    {
      "name": "approved",
      "ty": "Boolean"
    }
  ],
  "stream_response": null,
  "description": "Approves or rejects a liquidity update request for a given fund (admin role).",
  "json_schema": null
}"#;
}
impl WsResponse for AdminApproveLiquidityUpdateResponse {
    type Request = AdminApproveLiquidityUpdateRequest;
}

impl WsRequest for AdminListFundsWithPendingLiquidityRequest {
    type Response = AdminListFundsWithPendingLiquidityResponse;
    const METHOD_ID: u32 = 43000;
    const SCHEMA: &'static str = r#"{
  "name": "AdminListFundsWithPendingLiquidity",
  "code": 43000,
  "parameters": [],
  "returns": [
    {
      "name": "data",
      "ty": {
        "DataTable": {
          "name": "FundWithPendingLiquidity",
          "fields": [
            {
              "name": "fund_name",
              "ty": "String"
            },
            {
              "name": "pending_liquidity",
              "ty": "Numeric"
            },
            {
              "name": "fund_address",
              "ty": "String"
            },
            {
              "name": "target_apy",
              "ty": "Numeric"
            }
          ]
        }
      }
    }
  ],
  "stream_response": null,
  "description": "Returns a list of funds that have a non-None pending_liquidity (admin role).",
  "json_schema": null
}"#;
}
impl WsResponse for AdminListFundsWithPendingLiquidityResponse {
    type Request = AdminListFundsWithPendingLiquidityRequest;
}

impl WsRequest for CaptureLeadRequest {
    type Response = CaptureLeadResponse;
    const METHOD_ID: u32 = 43010;
    const SCHEMA: &'static str = r#"{
  "name": "CaptureLead",
  "code": 43010,
  "parameters": [
    {
      "name": "country",
      "ty": "String"
    },
    {
      "name": "email",
      "ty": "String"
    },
    {
      "name": "message",
      "ty": "String"
    },
    {
      "name": "name",
      "ty": "String"
    },
    {
      "name": "phone",
      "ty": "String"
    },
    {
      "name": "services_of_interest",
      "ty": {
        "EnumRef": "services_of_interest"
      }
    }
  ],
  "returns": [
    {
      "name": "lead_id",
      "ty": "BigInt"
    }
  ],
  "stream_response": null,
  "description": "Capture leads from the landing page.",
  "json_schema": null
}"#;
}
impl WsResponse for CaptureLeadResponse {
    type Request = CaptureLeadRequest;
}

impl WsRequest for AdminGetLeadsRequest {
    type Response = AdminGetLeadsResponse;
    const METHOD_ID: u32 = 43020;
    const SCHEMA: &'static str = r#"{
  "name": "AdminGetLeads",
  "code": 43020,
  "parameters": [],
  "returns": [
    {
      "name": "data",
      "ty": {
        "DataTable": {
          "name": "LeadModel",
          "fields": [
            {
              "name": "lead_id",
              "ty": "BigInt"
            },
            {
              "name": "name",
              "ty": "String"
            },
            {
              "name": "email",
              "ty": "String"
            },
            {
              "name": "phone",
              "ty": "String"
            },
            {
              "name": "country",
              "ty": "String"
            },
            {
              "name": "services_of_interest",
              "ty": {
                "EnumRef": "services_of_interest"
              }
            },
            {
              "name": "message",
              "ty": "String"
            },
            {
              "name": "created_at",
              "ty": "BigInt"
            }
          ]
        }
      }
    }
  ],
  "stream_response": null,
  "description": "Admin can view the list of captured leads with all fields (admin role).",
  "json_schema": null
}"#;
}
impl WsResponse for AdminGetLeadsResponse {
    type Request = AdminGetLeadsRequest;
}

impl WsRequest for UserConnectWalletRequest {
    type Response = UserConnectWalletResponse;
    const METHOD_ID: u32 = 43030;
    const SCHEMA: &'static str = r#"{
  "name": "UserConnectWallet",
  "code": 43030,
  "parameters": [
    {
      "name": "user_id",
      "ty": "BigInt"
    },
    {
      "name": "wallet_address",
      "ty": "String"
    },
    {
      "name": "wallet_type",
      "ty": {
        "EnumRef": "wallet_type"
      }
    }
  ],
  "returns": [
    {
      "name": "wallet_id",
      "ty": "BigInt"
    },
    {
      "name": "message",
      "ty": "String"
    }
  ],
  "stream_response": null,
  "description": "Connect a new wallet for a user",
  "json_schema": null
}"#;
}
impl WsResponse for UserConnectWalletResponse {
    type Request = UserConnectWalletRequest;
}

impl WsRequest for UserWalletRequest {
    type Response = UserWalletResponse;
    const METHOD_ID: u32 = 43040;
    const SCHEMA: &'static str = r#"{
  "name": "UserWallet",
  "code": 43040,
  "parameters": [
    {
      "name": "user_id",
      "ty": "BigInt"
    }
  ],
  "returns": [
    {
      "name": "wallets",
      "ty": {
        "DataTable": {
          "name": "UserWalletInfo",
          "fields": [
            {
              "name": "wallet_id",
              "ty": "BigInt"
            },
            {
              "name": "wallet_address",
              "ty": "String"
            },
            {
              "name": "wallet_type",
              "ty": {
                "EnumRef": "wallet_type"
              }
            },
            {
              "name": "balance",
              "ty": "Numeric"
            },
            {
              "name": "status",
              "ty": {
                "EnumRef": "wallet_status"
              }
            },
            {
              "name": "created_at",
              "ty": "BigInt"
            },
            {
              "name": "updated_at",
              "ty": "BigInt"
            }
          ]
        }
      }
    }
  ],
  "stream_response": null,
  "description": "Returns list of user's wallets",
  "json_schema": null
}"#;
}
impl WsResponse for UserWalletResponse {
    type Request = UserWalletRequest;
}

impl WsRequest for FundManagerManageAllocationsRequest {
    type Response = FundManagerManageAllocationsResponse;
    const METHOD_ID: u32 = 43050;
    const SCHEMA: &'static str = r#"{
  "name": "FundManagerManageAllocations",
  "code": 43050,
  "parameters": [
    {
      "name": "allocation_data",
      "ty": "String"
    },
    {
      "name": "fund_name",
      "ty": "String"
    }
  ],
  "returns": [
    {
      "name": "message",
      "ty": "String"
    }
  ],
  "stream_response": null,
  "description": "Manage the asset allocations of a fund.",
  "json_schema": null
}"#;
}
impl WsResponse for FundManagerManageAllocationsResponse {
    type Request = FundManagerManageAllocationsRequest;
}

impl WsRequest for FundManagerAdjustStrategyRequest {
    type Response = FundManagerAdjustStrategyResponse;
    const METHOD_ID: u32 = 43060;
    const SCHEMA: &'static str = r#"{
  "name": "FundManagerAdjustStrategy",
  "code": 43060,
  "parameters": [
    {
      "name": "fund_name",
      "ty": "String"
    },
    {
      "name": "new_strategy",
      "ty": {
        "EnumRef": "fund_strategy"
      }
    }
  ],
  "returns": [
    {
      "name": "new_strategy",
      "ty": {
        "EnumRef": "fund_strategy"
      }
    },
    {
      "name": "fund_name",
      "ty": "String"
    }
  ],
  "stream_response": null,
  "description": "Adjust the fund strategy.",
  "json_schema": null
}"#;
}
impl WsResponse for FundManagerAdjustStrategyResponse {
    type Request = FundManagerAdjustStrategyRequest;
}

impl WsRequest for FundManagerPerformanceReportRequest {
    type Response = FundManagerPerformanceReportResponse;
    const METHOD_ID: u32 = 43070;
    const SCHEMA: &'static str = r#"{
  "name": "FundManagerPerformanceReport",
  "code": 43070,
  "parameters": [
    {
      "name": "from_timestamp",
      "ty": "BigInt"
    },
    {
      "name": "fund_name",
      "ty": "String"
    },
    {
      "name": "to_timestamp",
      "ty": "BigInt"
    }
  ],
  "returns": [
    {
      "name": "report_url",
      "ty": "String"
    }
  ],
  "stream_response": null,
  "description": "Generates a fund performance report.",
  "json_schema": null
}"#;
}
impl WsResponse for FundManagerPerformanceReportResponse {
    type Request = FundManagerPerformanceReportRequest;
}

impl WsRequest for UserGetYieldDataRequest {
    type Response = UserGetYieldDataResponse;
    const METHOD_ID: u32 = 43080;
    const SCHEMA: &'static str = r#"{
  "name": "UserGetYieldData",
  "code": 43080,
  "parameters": [
    {
      "name": "from_timestamp",
      "ty": "BigInt"
    },
    {
      "name": "to_timestamp",
      "ty": "BigInt"
    },
    {
      "name": "user_id",
      "ty": "BigInt"
    }
  ],
  "returns": [
    {
      "name": "total_yield",
      "ty": "Numeric"
    },
    {
      "name": "daily_yield",
      "ty": "Numeric"
    },
    {
      "name": "annual_yield",
      "ty": "Numeric"
    }
  ],
  "stream_response": null,
  "description": "Retrieves yield data for a user.",
  "json_schema": null
}"#;
}
impl WsResponse for UserGetYieldDataResponse {
    type Request = UserGetYieldDataRequest;
}

impl WsRequest for AddYieldDataRequest {
    type Response = AddYieldDataResponse;
    const METHOD_ID: u32 = 43090;
    const SCHEMA: &'static str = r#"{
  "name": "AddYieldData",
  "code": 43090,
  "parameters": [
    {
      "name": "cagr",
      "ty": "Numeric"
    },
    {
      "name": "user_id",
      "ty": "BigInt"
    },
    {
      "name": "wallet_id",
      "ty": "BigInt"
    },
    {
      "name": "yield_amount",
      "ty": "Numeric"
    },
    {
      "name": "yield_date",
      "ty": "BigInt"
    }
  ],
  "returns": [
    {
      "name": "yield_id",
      "ty": "BigInt"
    },
    {
      "name": "message",
      "ty": "String"
    }
  ],
  "stream_response": null,
  "description": "Add yield data for a user.",
  "json_schema": null
}"#;
}
impl WsResponse for AddYieldDataResponse {
    type Request = AddYieldDataRequest;
}
