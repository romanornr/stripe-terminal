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

app.post('/cancel-latest-payment-intent', async (req, res) => {
    try {
        // Fetch the most recent Payment Intent
        const paymentIntents = await stripe.paymentIntents.list({
            limit: 1, // Get only the latest payment intent
        });

        if (!paymentIntents.data || paymentIntents.data.length === 0) {
            return res.status(404).json({ error: 'No payment intents found' });
        }

        const latestPaymentIntent = paymentIntents.data[0];

        // Attempt to cancel the Payment Intent
        try {
            const canceledPaymentIntent = await stripe.paymentIntents.cancel(latestPaymentIntent.id);
            return res.json(canceledPaymentIntent);
        } catch (error) {
            // Fetch the latest status if cancellation fails
            const updatedPaymentIntent = await stripe.paymentIntents.retrieve(latestPaymentIntent.id);
            return res.status(400).json({
                error: 'Payment intent is not in a cancelable state or could not be canceled',
                currentStatus: updatedPaymentIntent.status,
            });
        }
    } catch (error) {
        console.error('Error canceling or retrieving payment intent:', error);
        res.status(500).json({ error: 'Failed to cancel or retrieve the latest payment intent' });
    }
});

// cancel all payment intents
app.post('/cancel-all-payment-intents', async (req, res) => {
    try {
        // Fetch the most recent Payment Intent
        const paymentIntents = await stripe.paymentIntents.list({
            limit: 10,
        });

        if (!paymentIntents.data || paymentIntents.data.length === 0) {
            return res.status(404).json({error: 'No payment intents found'});
        }

        const listPaymentIntents = paymentIntents.data;

        // cancel all payment intents from listPaymentIntents
        for (let i = 0; i < listPaymentIntents.length; i++) {
            const paymentIntent = listPaymentIntents[i];
            try {
                const canceledPaymentIntent = await stripe.paymentIntents.cancel(paymentIntent.id);
                console.log('Payment intent canceled:', canceledPaymentIntent.id);
            } catch (error) {
                console.error('Error canceling payment intent:', error);
            }
        }

        // curl request to cancel all payment intents
        // curl -X POST https://localhost:4242/cancel-all-payment-intents -H "Content-Type: application/json" -d "{}"
    } catch (error) {
        console.error('Error canceling payment intents:', error);
        res.status(500).json({error: 'Failed to cancel payment intents'});
    }
});

app.post('/readers/cancel-action', async (req, res) => {
    try {
        const {reader_id} = req.body;
        const readerState = await stripe.terminal.readers.cancelAction(reader_id);
        res.send({ reader_state: readerState});
    } catch (error) {
        return res.status(400).send({ error: { message: error.message } });
    }
});
    

app.post('/connection-token', async (req, res) => {
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
