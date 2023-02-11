mod transformers;

use std::fmt::Debug;
use error_stack::{ResultExt, IntoReport};

use crate::{
    configs::settings,
    utils::{self, BytesExt},
    core::{
        errors::{self, CustomResult},
        payments,
    },
    headers, logger, services::{self, ConnectorIntegration},
    types::{
        self,
        api::{self, ConnectorCommon, ConnectorCommonExt},
        ErrorResponse, Response,
    }
};

use super::utils::RefundsRequestData;
use transformers as forte;

#[derive(Debug, Clone)]
pub struct Forte;

impl<Flow, Request, Response> ConnectorCommonExt<Flow, Request, Response> for Forte 
where
    Self: ConnectorIntegration<Flow, Request, Response>,{
    fn build_headers(
        &self,
        _req: &types::RouterData<Flow, Request, Response>,
        _connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, String)>, errors::ConnectorError> {
        // todo!()
        // let mut headers = vec![(
        //     headers::CONTENT_TYPE.to_string(),
        //     self.get_content_type().to_string(),
        // )];
        // let access_token = _req
        //     .access_token
        //     .clone()
        //     .ok_or(errors::ConnectorError::FailedToObtainAuthType)?;

        // let auth_header = (
        //     headers::AUTHORIZATION.to_string(),
        //     format!("Basic {}", access_token.token),
        // );

        // headers.push(auth_header);
        // Ok(headers)\
        Ok(vec![
            (
                headers::CONTENT_TYPE.to_string(),
                self.get_content_type().to_string(),
            ),
            ("X-Forte-Auth-Organization-Id".to_string(), "org_436911".to_string()),
            (
                headers::AUTHORIZATION.to_string(),
                // format!("Basic {}", access_token.token),
                format!("Basic {}", "Njk0M2NhZTVjNjdlZGFlNGMwNDQxNzcxNjFiMjM2ZTU6OGY5MmU1ZjY2ODY0ODBlMTNjZDlmOTU0Y2I4OGVmZWU="),
            ),
        ])
    }
}

impl ConnectorCommon for Forte {
    fn id(&self) -> &'static str {
        "forte"
    }

    fn common_get_content_type(&self) -> &'static str {
        // todo!()
        "application/json"
        // Ex: "application/x-www-form-urlencoded"
    }

    fn base_url<'a>(&self, connectors: &'a settings::Connectors) -> &'a str {
        connectors.forte.base_url.as_ref()
    }

    fn get_auth_header(&self, auth_type:&types::ConnectorAuthType)-> CustomResult<Vec<(String,String)>,errors::ConnectorError> {
        let auth: forte::ForteAuthType = auth_type
            .try_into()
            .change_context(errors::ConnectorError::FailedToObtainAuthType)?;
        Ok(vec![(headers::AUTHORIZATION.to_string(), auth.api_key)])
    }
}

impl api::Payment for Forte {}

impl api::PreVerify for Forte {}
impl
    ConnectorIntegration<
        api::Verify,
        types::VerifyRequestData,
        types::PaymentsResponseData,
    > for Forte
{
}

impl api::PaymentVoid for Forte {}

impl
    ConnectorIntegration<
        api::Void,
        types::PaymentsCancelData,
        types::PaymentsResponseData,
    > for Forte
{
    fn get_headers(
        &self,
        req: &types::PaymentsCancelRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, String)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        req: &types::PaymentsCancelRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        let connector_payment_id = &req.request.connector_transaction_id;
        let v :Vec<&str> = connector_payment_id.as_str().split(":_:").collect();
        let pp = format!(
            "{}/{}",
            self.base_url(connectors),
            v[0].to_string()
        );
        println!("something url--> {pp:?}");
        Ok(pp)
    }

    fn get_request_body(
        &self,
        _req: &types::PaymentsCancelRouterData,
    ) -> CustomResult<Option<String>, errors::ConnectorError> {
        // todo!()
        let connector_req = forte::FortePaymentsVoidRequest::try_from(_req)?;
        let forte_req = utils::Encode::<forte::FortePaymentsVoidRequest>::encode_to_string_of_json(
            &connector_req,
        )
        .change_context(errors::ConnectorError::RequestEncodingFailed)?;
        Ok(Some(forte_req))
    }

    fn build_request(
        &self,
        req: &types::PaymentsCancelRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        let request = services::RequestBuilder::new()
            .method(services::Method::Put)
            .url(&types::PaymentsVoidType::get_url(self, req, connectors)?)
            .headers(types::PaymentsVoidType::get_headers(self, req, connectors)?)
            .body(types::PaymentsVoidType::get_request_body(self, req)?)
            .build();
        Ok(Some(request))
    }
    fn handle_response(
        &self,
        data: &types::PaymentsCancelRouterData,
        res: Response,
    ) -> CustomResult<types::PaymentsCancelRouterData, errors::ConnectorError> {
        let response: forte::ForteResponse = res
            .response
            .parse_struct("Forte Response")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        logger::debug!(forte_create_response=?response);
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }
    fn get_error_response(
        &self,
        res: types::Response,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res)
    }
}

impl api::ConnectorAccessToken for Forte {}

impl ConnectorIntegration<api::AccessTokenAuth, types::AccessTokenRequestData, types::AccessToken>
    for Forte
{
}

impl api::PaymentSync for Forte {}
impl
    ConnectorIntegration<api::PSync, types::PaymentsSyncData, types::PaymentsResponseData>
    for Forte
{
    fn get_headers(
        &self,
        req: &types::PaymentsSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, String)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        _req: &types::PaymentsSyncRouterData,
        _connectors: &settings::Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        let connector_payment_id = _req
            .request
            .connector_transaction_id
            .get_connector_transaction_id()
            .change_context(errors::ConnectorError::MissingConnectorTransactionID)?;
        Ok(
            format!(
            "{}/trn_{}",
            self.base_url(_connectors),
            connector_payment_id
        ))
    }

    fn build_request(
        &self,
        req: &types::PaymentsSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Get)
                .url(&types::PaymentsSyncType::get_url(self, req, connectors)?)
                .headers(types::PaymentsSyncType::get_headers(self, req, connectors)?)
                .build(),
        ))
    }

    fn get_error_response(
        &self,
        res: Response,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res)
    }

    fn handle_response(
        &self,
        data: &types::PaymentsSyncRouterData,
        res: Response,
    ) -> CustomResult<types::PaymentsSyncRouterData, errors::ConnectorError> {
        logger::debug!(payment_sync_response=?res);
        let response: forte:: FortePaymentsResponse = res
            .response
            .parse_struct("forte PaymentsResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }
}


impl api::PaymentCapture for Forte {}
impl
    ConnectorIntegration<
        api::Capture,
        types::PaymentsCaptureData,
        types::PaymentsResponseData,
    > for Forte
{
    fn get_headers(
        &self,
        req: &types::PaymentsCaptureRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Vec<(String, String)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        _req: &types::PaymentsCaptureRouterData,
        _connectors: &settings::Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        // todo!()
        Ok(self.base_url(_connectors).to_string())
    }

    fn get_request_body(
        &self,
        _req: &types::PaymentsCaptureRouterData,
    ) -> CustomResult<Option<String>, errors::ConnectorError> {
        // todo!()
        let connector_req = forte::FortePaymentsCaptureRequest::try_from(_req)?;
        let forte_req = utils::Encode::<forte::FortePaymentsCaptureRequest>::encode_to_string_of_json(
            &connector_req,
        )
        .change_context(errors::ConnectorError::RequestEncodingFailed)?;
        Ok(Some(forte_req))
        
    }

    fn build_request(
        &self,
        req: &types::PaymentsCaptureRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Put)
                .url(&types::PaymentsCaptureType::get_url(self, req, connectors)?)
                .headers(types::PaymentsCaptureType::get_headers(
                    self, req, connectors,
                )?)
                .body(types::PaymentsCaptureType::get_request_body(self, req)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::PaymentsCaptureRouterData,
        res: Response,
    ) -> CustomResult<types::PaymentsCaptureRouterData, errors::ConnectorError> {

        // let type_a = Type_A::try_from(type_b)?;

        // let type_a: Type_A = type_b.try_into()?;


        let response: forte::ForteResponse = res
            .response
            .parse_struct("Forte Response")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        // println!("something3 {}",response);
        logger::debug!(forte_create_response=?response);
        types::RouterData::try_from(types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
        // types::ResponseRouterData {
        //     response,
        //     data: data.clone(),
        //     http_code: res.status_code,
        // }
        // .try_into()
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response(
        &self,
        res: Response,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res)
    }
}

impl api::PaymentSession for Forte {}

impl
    ConnectorIntegration<
        api::Session,
        types::PaymentsSessionData,
        types::PaymentsResponseData,
    > for Forte
{
    //TODO: implement sessions flow
}

impl api::PaymentAuthorize for Forte {}

impl
    ConnectorIntegration<
        api::Authorize,
        types::PaymentsAuthorizeData,
        types::PaymentsResponseData,
    > for Forte {
    fn get_headers(&self, req: &types::PaymentsAuthorizeRouterData, connectors: &settings::Connectors,) -> CustomResult<Vec<(String, String)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(&self, _req: &types::PaymentsAuthorizeRouterData, connectors: &settings::Connectors,) -> CustomResult<String,errors::ConnectorError> {
        let capture_method = _req.request.capture_method;
        match capture_method {
            Some(storage_models::enums::CaptureMethod::Automatic) => Ok(format!("{}/sale", self.base_url(connectors))),
            _ => Ok(format!("{}/authorize", self.base_url(connectors)))
        }
    }

    fn get_request_body(&self, req: &types::PaymentsAuthorizeRouterData) -> CustomResult<Option<String>,errors::ConnectorError> {
        let forte_req =
            utils::Encode::<forte::FortePaymentsRequest>::convert_and_encode(req).change_context(errors::ConnectorError::RequestEncodingFailed)?;
        Ok(Some(forte_req))
    }

    fn build_request(
        &self,
        req: &types::PaymentsAuthorizeRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Post)
                .url(&types::PaymentsAuthorizeType::get_url(
                    self, req, connectors,
                )?)
                .headers(types::PaymentsAuthorizeType::get_headers(
                    self, req, connectors,
                )?)
                .body(types::PaymentsAuthorizeType::get_request_body(self, req)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::PaymentsAuthorizeRouterData,
        res: Response,
    ) -> CustomResult<types::PaymentsAuthorizeRouterData,errors::ConnectorError> {
        let response: forte::FortePaymentsResponse = res
            .response
            .parse_struct("FortePaymentsResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        logger::debug!(fortepayments_create_response=?response);
        types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        }
        .try_into()
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response(&self, res: Response) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res)
    }
}

impl api::Refund for Forte {}
impl api::RefundExecute for Forte {}
impl api::RefundSync for Forte {}

impl
    ConnectorIntegration<
        api::Execute,
        types::RefundsData,
        types::RefundsResponseData,
    > for Forte {
    fn get_headers(&self, req: &types::RefundsRouterData<api::Execute>, connectors: &settings::Connectors,) -> CustomResult<Vec<(String,String)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(&self, _req: &types::RefundsRouterData<api::Execute>, _connectors: &settings::Connectors,) -> CustomResult<String,errors::ConnectorError> {
        // todo!()
        Ok(format!("{}", self.base_url(_connectors),))
    }

    fn get_request_body(&self, req: &types::RefundsRouterData<api::Execute>) -> CustomResult<Option<String>,errors::ConnectorError> {
        let forte_req = utils::Encode::<forte::ForteRefundRequest>::convert_and_encode(req).change_context(errors::ConnectorError::RequestEncodingFailed)?;
        Ok(Some(forte_req))
    }

    fn build_request(&self, req: &types::RefundsRouterData<api::Execute>, connectors: &settings::Connectors,) -> CustomResult<Option<services::Request>,errors::ConnectorError> {
        let request = services::RequestBuilder::new()
            .method(services::Method::Post)
            .url(&types::RefundExecuteType::get_url(self, req, connectors)?)
            .headers(types::RefundExecuteType::get_headers(self, req, connectors)?)
            .body(types::RefundExecuteType::get_request_body(self, req)?)
            .build();
        Ok(Some(request))
    }

    fn handle_response(
        &self,
        data: &types::RefundsRouterData<api::Execute>,
        res: Response,
    ) -> CustomResult<types::RefundsRouterData<api::Execute>,errors::ConnectorError> {
        logger::debug!(target: "router::connector::forte", response=?res);
        let response: forte::RefundResponse = res.response.parse_struct("forte RefundResponse").change_context(errors::ConnectorError::RequestEncodingFailed)?;
        types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        }
        .try_into()
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response(&self, res: Response) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res)
    }
}

impl
    ConnectorIntegration<api::RSync, types::RefundsData, types::RefundsResponseData> for Forte {
    fn get_headers(&self, req: &types::RefundSyncRouterData,connectors: &settings::Connectors,) -> CustomResult<Vec<(String, String)>,errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url( &self,
        _req: &types::RefundSyncRouterData,
        _connectors: &settings::Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        let refund_id = _req.request.get_connector_refund_id()?;
        let rp = 
            format!(
            "{}/trn_{}",
            self.base_url(_connectors),
            refund_id
        );
        println!("refund req: {}",rp);
        Ok(rp)
    }

    fn build_request(
        &self,
        req: &types::RefundSyncRouterData,
        connectors: &settings::Connectors,
    ) -> CustomResult<Option<services::Request>, errors::ConnectorError> {
        Ok(Some(
            services::RequestBuilder::new()
                .method(services::Method::Get)
                .url(&types::RefundSyncType::get_url(self, req, connectors)?)
                .headers(types::RefundSyncType::get_headers(self, req, connectors)?)
                .body(types::RefundSyncType::get_request_body(self, req)?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &types::RefundSyncRouterData,
        res: Response,
    ) -> CustomResult<types::RefundSyncRouterData,errors::ConnectorError,> {
        logger::debug!(target: "router::connector::forte", response=?res);
        let response: forte::RefundResponse = res.response.parse_struct("forte RefundResponse").change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        types::ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        }
        .try_into()
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response(&self, res: Response) -> CustomResult<ErrorResponse,errors::ConnectorError> {
        self.build_error_response(res)
    }
}

#[async_trait::async_trait]
impl api::IncomingWebhook for Forte {
    fn get_webhook_object_reference_id(
        &self,
        _body: &[u8],
    ) -> CustomResult<String, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }

    fn get_webhook_event_type(
        &self,
        _body: &[u8],
    ) -> CustomResult<api::IncomingWebhookEvent, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }

    fn get_webhook_resource_object(
        &self,
        _body: &[u8],
    ) -> CustomResult<serde_json::Value, errors::ConnectorError> {
        Err(errors::ConnectorError::WebhooksNotImplemented).into_report()
    }
}

impl services::ConnectorRedirectResponse for Forte {
    fn get_flow_type(
        &self,
        _query_params: &str,
    ) -> CustomResult<payments::CallConnectorAction, errors::ConnectorError> {
        Ok(payments::CallConnectorAction::Trigger)
    }
}
