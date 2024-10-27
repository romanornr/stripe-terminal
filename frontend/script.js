console.log("Script loaded");

document.getElementById('payButton').addEventListener('click', async () => {
	const amountInput = document.getElementById('amount');
	const amountInCents = Math.round(parseFloat(amountInput.value) * 100); // Convert to cents
    
	if (!amountInCents || amountInCents <= 50) {
	    alert("Please enter a valid amount above 0.50");
	    return;
	}
    
	try {
	    // Step 1: Call backend to create a Payment Intent
	    const response = await fetch('http://localhost:4242/create-payment-intent', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ amount: amountInCents })
	    });
	    
	    const { clientSecret } = await response.json();
    
	    // Step 2: Initialize Stripe Terminal and get a connection token
	    const terminal = StripeTerminal.create({
		onFetchConnectionToken: async () => {
		    const tokenResponse = await fetch('http://localhost:4242/connection_token', {
			method: 'POST'
		    });
		    const { secret } = await tokenResponse.json();
		    return secret;
		},
		onUnexpectedReaderDisconnect: () => {
			alert("The reader was disconnected unexpectedly. Please reconnect.");
			console.error("Reader disconnected unexpectedly");
		}		
	    });
    
	    // Discover and connect to the terminal reader
	    console.log("Discovering reader");

	    const locationResponse = await fetch('http://localhost:4242/get-location-id');
	    const { locationId } = await locationResponse.json();

	    const discoverResult = await terminal.discoverReaders({ location: locationId });
	    console.log("Discover result:", discoverResult);

	    if (discoverResult.error) {
		console.error("Error discovering reader:", discoverResult.error.message);
		alert("Error discovering reader. Please check the connection and try again.");
		return;
	    }

	    if (!discoverResult.discoveredReaders || !Array.isArray(discoverResult.discoveredReaders) || discoverResult.discoveredReaders.length === 0) {
		console.log("No readers found:", discoverResult.discoveredReaders);
		alert("No readers found. Please make sure the reader is available and try again.");
		return;
	    }

	    // Connect to the first reader found
	    const reader = discoverResult.discoveredReaders[0];
	    console.log("Connecting to reader:", reader);

	    const connectResult = await terminal.connectReader(reader);

	    if (connectResult.error) {
		console.error("Error connecting to reader:", connectResult.error.message);
		return;
	    }

	    console.log("Reader connected successfully!");

	    // Step 3: Collect Payment Method on the connected reader
	    console.log("Collecting payment method...");

	    const collectResult = await terminal.collectPaymentMethod(clientSecret);

	    if (collectResult.error) {
	        console.error("Error collecting payment method:", collectResult.error.message);
	        alert("Error collecting payment. Please try again.");
	        return;
	    }

	    const paymentIntent = collectResult.paymentIntent;
	    if (!paymentIntent) {
	        console.error("Error collecting payment method: No Payment Intent returned");
	        alert("Error collecting payment. Please try again.");
	        return;
	    }

	    // Step 4: Process the Payment after collecting the payment method
	    console.log("Processing payment...");

	    const processResult = await terminal.processPayment(paymentIntent);

	    if (processResult.paymentIntent && processResult.paymentIntent.status === 'succeeded') {
	        alert("Payment successful!");
	        console.log("Payment successful:", processResult.paymentIntent);
	    } else if (processResult.error) {
	        console.error("Error processing payment:", processResult.error.message);
	        alert("Payment failed. Please try again.");
	    }

	} catch (error) {
		console.error("Error:", error);
		alert("An error occurred. Please try again.");
	}
});
