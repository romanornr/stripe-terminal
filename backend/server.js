require('dotenv').config();
const express = require('express');
const cors = require('cors'); // Import cors
const stripe = require('stripe')(process.env.STRIPE_SECRET_KEY);
const app = express();
app.use(express.json());
app.use(cors()); // Enable CORS for all routes

app.post('/create-payment-intent', async (req, res) => {
    const {amount} = req.body;
    const paymentIntent = await stripe.paymentIntents.create({
        amount,
        currency: 'eur',
        payment_method_types: ['card_present'],
        capture_method: 'automatic',
    });
    res.json({clientSecret: paymentIntent.client_secret});
});

app.get('/get-recent-payments', async (req, res) => {
    try {
        // Fetch the 10 most recent payments from Stripe
        const payments = await stripe.paymentIntents.list({
            limit: 10,
        });

        res.json({payments: payments.data});
        } catch (error) {
            console.error("Error fetching recent payments:", error);
            res.status(500).json({error: 'Error fetching recent payments'});
        }
    });

app.post('/connection_token', async (req, res) => {
    try {
        const connectionToken = await stripe.terminal.connectionTokens.create();
        res.json({secret: connectionToken.secret});
    } catch (error) {
        console.error("Error creating connection token:", error);
        res.status(500).json({error: 'Error creating connection token'});
    }
});

app.get('/get-location-id', (req, res) => {
    res.json({locationId: process.env.STRIPE_TERMINAL_LOCATION_ID});
})

app.listen(4242, () => console.log('Server running on port 4242'));
