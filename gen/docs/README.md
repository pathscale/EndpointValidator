
# auth Server
ID: 1
## Endpoints
|Method Code|Method Name|Parameters|Response|Description|
|-----------|-----------|----------|--------|-----------|
|10020|Login|username, password|username, display_name, avatar, role, user_id, user_token, admin_token|User login|
|10021|SignUp|password, username|username, display_name, avatar, role, user_id, user_token, admin_token|User registration|

# user Server
ID: 2
## Endpoints
|Method Code|Method Name|Parameters|Response|Description|
|-----------|-----------|----------|--------|-----------|
|40000|UserPing||pong|A simple health check: receives 'ping' and returns 'pong'.|
|41000|UserGetTopFunds||data|Fetch the top 3 funds by liquidity or other ranking metric.|
|41010|UserGetFundDetails|fund_name|fund_name, fund_liquidity, fund_target_apy, fund_address, fund_start_date, description|Return details for the specified fund.|
|41020|UserListFunds||data|List all active funds.|
|41030|FundManagerSignup|allocation_data, aum, company_name, country, description, fund_address, fund_name, fund_start_date, fund_strategy, fund_target_apy, last_updated_at, legal_documents, manager_company_name, manager_phone_number, manager_email, strategy_description, performance_reports|fund_id, message|Allow a user to register a new fund.|
|41040|FundManagerUpdateLiquidity|fund_name, new_liquidity||Update the specified fund's liquidity (f64).|
|42010|AdminListPendingFunds||data|List funds with pending status (admin role).|
|42020|AdminListUsers||data|List all users, including name, email, and role (admin role).|
|42030|AdminApproveRejectFund|comment, fund_name, new_status|fund_name, new_status, comment, legal_documents|Approve or reject a specified fund (admin role).|
|42040|AdminApproveLiquidityUpdate|fund_name, approve|fund_name, old_liquidity, new_liquidity, approved|Approves or rejects a liquidity update request for a given fund (admin role).|
|43000|AdminListFundsWithPendingLiquidity||data|Returns a list of funds that have a non-None pending_liquidity (admin role).|
|43010|CaptureLead|country, email, message, name, phone, services_of_interest|lead_id|Capture leads from the landing page.|
|43020|AdminGetLeads||data|Admin can view the list of captured leads with all fields (admin role).|
|43030|UserConnectWallet|user_id, wallet_address, wallet_type|wallet_id, message|Connect a new wallet for a user|
|43040|UserWallet|user_id|wallets|Returns list of user's wallets|
|43050|FundManagerManageAllocations|allocation_data, fund_name|message|Manage the asset allocations of a fund.|
|43060|FundManagerAdjustStrategy|fund_name, new_strategy|new_strategy, fund_name|Adjust the fund strategy.|
|43070|FundManagerPerformanceReport|from_timestamp, fund_name, to_timestamp|report_url|Generates a fund performance report.|
|43080|UserGetYieldData|from_timestamp, to_timestamp, user_id|total_yield, daily_yield, annual_yield|Retrieves yield data for a user.|
|43090|AddYieldData|cagr, user_id, wallet_id, yield_amount, yield_date|yield_id, message|Add yield data for a user.|
