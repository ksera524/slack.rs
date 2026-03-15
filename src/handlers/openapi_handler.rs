use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::Response,
};

const OPENAPI_JSON: &str = r##"{
  "openapi": "3.0.3",
  "info": {
    "title": "api-hub API",
    "version": "0.1.0",
    "description": "Slack APIおよびS3互換ストレージ操作を提供するAPI"
  },
  "servers": [
    {
      "url": "http://localhost:3000"
    }
  ],
  "paths": {
    "/health": {
      "get": {
        "summary": "Health check",
        "responses": {
          "200": {
            "description": "Service is healthy",
            "content": {
              "text/plain": {
                "schema": {
                  "type": "string",
                  "example": "ok"
                }
              }
            }
          }
        }
      }
    },
    "/slack/message": {
      "post": {
        "summary": "Post a text message",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/SlackMessageRequest"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Raw Slack API response JSON string",
            "content": {
              "application/json": {
                "schema": {
                  "type": "string"
                }
              }
            }
          },
          "400": {
            "$ref": "#/components/responses/BadRequest"
          },
          "500": {
            "$ref": "#/components/responses/InternalServerError"
          }
        }
      }
    },
    "/slack/upload/image": {
      "post": {
        "summary": "Upload an image to a channel",
        "parameters": [
          {
            "name": "channel",
            "in": "query",
            "required": true,
            "schema": {
              "type": "string"
            }
          },
          {
            "name": "file_name",
            "in": "query",
            "required": false,
            "schema": {
              "type": "string"
            }
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "image/png": {
              "schema": {
                "type": "string",
                "format": "binary"
              }
            },
            "image/jpeg": {
              "schema": {
                "type": "string",
                "format": "binary"
              }
            },
            "image/webp": {
              "schema": {
                "type": "string",
                "format": "binary"
              }
            },
            "image/gif": {
              "schema": {
                "type": "string",
                "format": "binary"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Raw Slack API response JSON string",
            "content": {
              "application/json": {
                "schema": {
                  "type": "string"
                }
              }
            }
          },
          "400": {
            "$ref": "#/components/responses/BadRequest"
          },
          "500": {
            "$ref": "#/components/responses/InternalServerError"
          }
        }
      }
    },
    "/slack/upload/pdf": {
      "post": {
        "summary": "Upload a PDF to a channel",
        "parameters": [
          {
            "name": "channel",
            "in": "query",
            "required": true,
            "schema": {
              "type": "string"
            }
          },
          {
            "name": "file_name",
            "in": "query",
            "required": false,
            "schema": {
              "type": "string"
            }
          }
        ],
        "requestBody": {
          "required": true,
          "content": {
            "application/pdf": {
              "schema": {
                "type": "string",
                "format": "binary"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Raw Slack API response JSON string",
            "content": {
              "application/json": {
                "schema": {
                  "type": "string"
                }
              }
            }
          },
          "400": {
            "$ref": "#/components/responses/BadRequest"
          },
          "500": {
            "$ref": "#/components/responses/InternalServerError"
          }
        }
      }
    }
  },
  "components": {
    "responses": {
      "BadRequest": {
        "description": "Bad request",
        "content": {
          "application/problem+json": {
            "schema": {
              "$ref": "#/components/schemas/ProblemDetails"
            }
          }
        }
      },
      "InternalServerError": {
        "description": "Internal server error",
        "content": {
          "application/problem+json": {
            "schema": {
              "$ref": "#/components/schemas/ProblemDetails"
            }
          }
        }
      }
    },
    "schemas": {
      "SlackMessageRequest": {
        "type": "object",
        "properties": {
          "channel": {
            "type": "string"
          },
          "text": {
            "type": "string"
          }
        },
        "required": ["channel", "text"]
      },
      "ProblemDetails": {
        "type": "object",
        "properties": {
          "type": {
            "type": "string",
            "example": "about:blank"
          },
          "title": {
            "type": "string",
            "example": "Bad Request"
          },
          "status": {
            "type": "integer",
            "format": "int32",
            "example": 400
          },
          "detail": {
            "type": "string"
          }
        },
        "required": ["type", "title", "status", "detail"]
      }
    }
  }
}"##;

pub async fn openapi_json() -> Response {
    let mut response = Response::new(Body::from(OPENAPI_JSON));
    *response.status_mut() = StatusCode::OK;
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}
