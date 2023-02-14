use serde::{Deserialize, Serialize};
use crate::{core::errors,pii::PeekInterface,types::{self,api, storage::enums}};

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize, PartialEq)]
pub struct FortePaymentsRequest {
    authorization_amount : f32,
    subtotal_amount : f32, 
    billing_address : BillingInfo,
    card : Card,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct BillingInfo {
    first_name: String,
    last_name: String,
}

#[derive(Default, Debug, Serialize, Eq, PartialEq)]
pub struct Card {
    card_type: String,
    name_on_card: String,
    account_number: String,
    expire_month: String,
    expire_year: String,
    card_verification_value: String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
// #[serde(rename_all = "camelCase")]
pub struct Card2 {
    card_type: String,
    name_on_card: String,
    last_4_account_number: String,
    masked_account_number: String,
    expire_month: i32,
    expire_year: i32
}

impl TryFrom<&types::PaymentsAuthorizeRouterData> for FortePaymentsRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsAuthorizeRouterData) -> Result<Self,Self::Error> {
        // todo!()
        match item.request.payment_method_data {
            api::PaymentMethod::Card(ref ccard) => {
                let payment_request = Self {
                    authorization_amount: item.request.amount as f32,
                    subtotal_amount: item.request.amount as f32,
                    billing_address : BillingInfo { first_name: "saurav".to_string(), last_name: "cv".to_string() },
                    card: Card {
                        card_type: "visa".to_string(),
                        name_on_card: ccard.card_holder_name.peek().clone(),
                        account_number: ccard.card_number.peek().clone(),
                        expire_month: ccard.card_exp_month.peek().clone(),
                        expire_year: ccard.card_exp_year.peek().clone(),
                        card_verification_value : ccard.card_cvc.peek().clone(),
                    },
                };

                println!("something --> {payment_request:?}");
                let tmp = serde_json::to_string(&payment_request);
                println!("something2 --> {tmp:?}");
                Ok(payment_request)
            }
            _ => Err(
                errors::ConnectorError::NotImplemented("Current Payment Method".to_string()).into(),
            ),
    }
}
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct ForteAuthType {
    pub(super) api_key: String
}

impl TryFrom<&types::ConnectorAuthType> for ForteAuthType  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(_auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        todo!()
    }
}
// PaymentsResponse
//TODO: Append the remaining status flags
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FortePaymentStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<FortePaymentStatus> for enums::AttemptStatus {
    fn from(item: FortePaymentStatus) -> Self {
        match item {
            FortePaymentStatus::Succeeded => Self::Charged,
            FortePaymentStatus::Failed => Self::Failure,
            FortePaymentStatus::Processing => Self::Authorizing,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Response {
    environment: String,
    response_type: String,
    response_code: String,
    response_desc: String,
    authorization_code: String,
    avs_result: String,
    cvv_result: String
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FortePaymentsResponse {
    // status: FortePaymentStatus,
    // id: String,
        transaction_id: String,
        location_id: String,
        action: String,
        authorization_amount: f32,
        entered_by: String,
        billing_address: BillingInfo,
        card: Card2,
        response: Response
      }

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Desc {
    pub response_desc: String
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorDesc {
    pub response : Desc
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DefaultResponse {
    pub response_type : String,
    pub response_code:String,
    pub response_desc:String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ForteResponse {
    transaction_id: String,
    action: String,
    pub response: DefaultResponse
}

fn get_payment_status(resp: FortePaymentsResponse) -> enums::AttemptStatus {
    match (resp.action.as_str(), resp.response.response_code.as_str()) {
        ("sale","A01") => enums::AttemptStatus::Charged,
        ("authorize","A01") => enums::AttemptStatus::Authorized,
        ("capture", "A01") => enums::AttemptStatus::Charged,
        ("void", "A01") => enums::AttemptStatus::Voided,
        (_, _) => enums::AttemptStatus::Failure

    }
}

impl<F,T> TryFrom<types::ResponseRouterData<F, FortePaymentsResponse, T, types::PaymentsResponseData>> for types::RouterData<F, T, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(item: types::ResponseRouterData<F, FortePaymentsResponse, T, types::PaymentsResponseData>) -> Result<Self,Self::Error> {
        let mut txn_id= String::from(item.response.clone().transaction_id);
        txn_id.push_str(":_:");
        txn_id.push_str(item.response.response.authorization_code.as_str());
        txn_id.push_str(":_:");
        txn_id.push_str(item.response.entered_by.as_str());
        Ok(Self {
            status: enums::AttemptStatus::from(get_payment_status(item.response)),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(txn_id),
                redirection_data: None,
                redirect: false,
                mandate_reference: None,
                connector_metadata: None,
            }),
            ..item.data
        })
    }
}

fn get_payment_status2(resp: ForteResponse) -> enums::AttemptStatus {
    match (resp.action.as_str(), resp.response.response_code.as_str()) {
        ("sale","A01") => enums::AttemptStatus::Charged,
        ("authorize","A01") => enums::AttemptStatus::Authorized,
        ("capture", "A01") => enums::AttemptStatus::Charged,
        ("void", "A01") => enums::AttemptStatus::Voided,
        (_, _) => enums::AttemptStatus::Failure

    }
}

impl<F,T> TryFrom<types::ResponseRouterData<F, ForteResponse, T, types::PaymentsResponseData>> for types::RouterData<F, T, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(item: types::ResponseRouterData<F, ForteResponse, T, types::PaymentsResponseData>) -> Result<Self,Self::Error> {
        Ok(Self {
            status: enums::AttemptStatus::from(get_payment_status2(item.response.clone())),
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.transaction_id),
                redirection_data: None,
                redirect: false,
                mandate_reference: None,
                connector_metadata: None,
            }),
            ..item.data
        })
    }
}

#[derive(Default, Debug, Clone, Serialize, PartialEq)]
pub struct FortePaymentsCaptureRequest {
    action: String,
    transaction_id: String,
    authorization_code : String
}

impl TryFrom<&types::PaymentsCaptureRouterData> for FortePaymentsCaptureRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsCaptureRouterData) -> Result<Self, Self::Error> {
        let v :Vec<&str> = item.request.connector_transaction_id.as_str().split(":_:").collect();
        let t_id = if v.len()>0 {v[0]} else {""};
        let auth_code = if v.len()>1 {v[1]} else {""};
        let pp = Self {
            action: "capture".to_string(),
            transaction_id: t_id.to_string(),
            authorization_code: auth_code.to_string(),
        };
        println!("something 4--> {pp:?}");
        let tmp = serde_json::to_string(&pp);
        println!("something 5 --> {tmp:?}");
    Ok(pp)
    }
}






#[derive(Default, Debug, Clone, Serialize, PartialEq)]
pub struct FortePaymentsVoidRequest {
    action: String,
    authorization_code: String,
    entered_by: String
}

impl TryFrom<&types::PaymentsCancelRouterData> for FortePaymentsVoidRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &types::PaymentsCancelRouterData) -> Result<Self, Self::Error> {
        let v :Vec<&str> = item.request.connector_transaction_id.as_str().split(":_:").collect();
        let enter_by = if v.len()>2 {v[2]} else {""};
        let auth_code = if v.len()>1 {v[1]} else {""};
        let pp = Self {
            action: "void".to_string(),
            authorization_code: auth_code.to_string(),
            entered_by: enter_by.to_string()
        };
        println!("something 4--> {pp:?}");
        let tmp = serde_json::to_string(&pp);
        println!("something 5 --> {tmp:?}");
    Ok(pp)
    }
}










//TODO: Fill the struct with respective fields
// REFUND :
// Type definition for RefundRequest

#[derive(Default, Debug, Serialize)]
pub struct ForteRefundRequest {
    action : String, 
    authorization_amount:f32,
    original_transaction_id:String,
    authorization_code:String
}

impl<F> TryFrom<&types::RefundsRouterData<F>> for ForteRefundRequest {
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: &types::RefundsRouterData<F>) -> Result<Self,Self::Error> {
    //    todo!()
    let v :Vec<&str> = _item.request.connector_transaction_id.as_str().split(":_:").collect();
    let t_id = if v.len()>0 {v[0]} else {""};
    let auth_code = if v.len()>1 {v[1]} else {""};
    Ok(Self {
        action: "reverse".to_string() ,
        authorization_amount: _item.request.refund_amount as f32,
        original_transaction_id : t_id.to_string(),
        authorization_code : auth_code.to_string()
    })
    
    }
    // println!("something --> {payment_request:?}");
    // let tmp = serde_json::to_string(&payment_request);
    // println!("something2 --> {tmp:?}");
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
pub enum ForteStatus {
    F01,
    N01,
    #[default]
    A01,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<ForteStatus> for enums::RefundStatus {
    fn from(item: ForteStatus) -> Self {
        match item {
            ForteStatus::A01 => Self::Success,
            ForteStatus::F01 => Self::Failure,
            ForteStatus::N01 => Self::Pending,
            //TODO: Review mapping
        }
    }
}


#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub struct ForteRefundResponse {
    response_type : String, 
    response_code:ForteStatus,
    response_desc:String,
}

//TODO: Fill the struct with respective fields
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub struct RefundResponse {
    transaction_id : String,
    response: ForteRefundResponse
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
    for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(
        _item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        // todo!()
        // let refund_status = RefundStatus::Succeeded;
        let refund_status = enums::RefundStatus::from(_item.response.response.response_code);
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: _item.response.transaction_id,
                refund_status,
            }),
            .._item.data
        })

    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, RefundResponse>> for types::RefundsRouterData<api::RSync>
{
     type Error = error_stack::Report<errors::ParsingError>;
    fn try_from(_item: types::RefundsResponseRouterData<api::RSync, RefundResponse>) -> Result<Self,Self::Error> {
         todo!()
     }
 }

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForteErrorResponse {}
