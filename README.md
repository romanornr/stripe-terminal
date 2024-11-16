# Stripe Terminal Backend

A backend service for handling Stripe Terminal payments, providing endpoints for payment processing, terminal connection management, and payment status tracking. This project includes both a Node.js implementation and a Rust implementation, allowing for flexible deployment options including potential future deployment to Cloudflare Workers.

## Overview

This backend service integrates with Stripe Terminal to facilitate in-person payments, providing essential functionality for:
- Creating payment intents for card-present transactions
- Managing terminal connections
- Tracking payment statuses
- Retrieving recent payment history
- Cancelling pending payment intents

## Project Structure

```
project-root/
│
├── backend/           # Rust implementation
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   └── .env
│
└── README.md
```

## Features

- **Payment Intent Creation**: Create new payment intents for card-present transactions
- **Recent Payments**: Retrieve the 10 most recent payment transactions
- **Payment Cancellation**: Cancel the most recent payment intent if it's in a cancellable state
- **Terminal Connection**: Generate connection tokens for Stripe Terminal devices
- **Location Management**: Retrieve configured terminal location IDs
- **CORS Support**: Enabled for cross-origin requests
- **Error Handling**: Comprehensive error handling and status reporting

## API Endpoints

- `POST /create-payment-intent`: Create a new payment intent
- `GET /get-recent-payments`: Retrieve recent payment history
- `POST /cancel-latest-payment-intent`: Cancel the most recent payment intent
- `POST /connection_token`: Generate a new terminal connection token
- `GET /get-location-id`: Get the configured terminal location ID

## Setup

### Prerequisites

- Stripe account with Terminal enabled
- For Node.js version:
  - Node.js 14+ installed
- For Rust version:
  - Rust toolchain installed (rustc, cargo)

### Environment Variables

Create a `.env` file in the project root with:

```env
STRIPE_SECRET_KEY=sk_your_secret_key
STRIPE_PUBLISHABLE_KEY=pk_your_publishable_key
STRIPE_TERMINAL_LOCATION_ID=tml_your_location_id
```

### Running the backend

1. Navigate to the backend directory:
```bash
cd backend
```

2. Install dependencies and run:
```bash
cargo run
```

The server will start on port 4242 for both versions.

## Implementation Details

### Rust Version
- Built with Actix-web framework
- Uses stripe-rust crate for Stripe API integration
- Asynchronous request handling
- Strong type safety and error handling
- Prepared for potential Cloudflare Workers deployment

## Deployment

Both implementations can be deployed to traditional hosting platforms. The Rust version is specifically designed with future Cloudflare Workers deployment in mind.

## Error Handling

The API returns appropriate HTTP status codes:
- 200: Successful operation
- 400: Invalid request
- 404: Resource not found
- 500: Server error

Error responses include descriptive messages in the response body.

## Security

- Environment variables for sensitive credentials
- CORS configuration for controlled access
- Secure connection token generation
- Proper error handling to prevent information leakage

## Future Enhancements

- Migration to Cloudflare Workers using the Rust implementation
- Enhanced monitoring and logging
- Additional payment features and integrations
- Performance optimizations


## License

MIT License