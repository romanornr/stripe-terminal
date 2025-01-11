console.log("Script loaded");

document.getElementById('cancelButton').addEventListener('click', async () => {
	// const { cancelAction } = await cancelReaderAction('tmr_Fxpdsw9DI1qvaP');
	// // if (cancelAction.error) {
	// // 	console.error("Error canceling reader action:", cancelAction.error);
	// // 	alert("Error canceling reader action. Please try again.");
	// // 	cancelButton.disabled = false;
	// // 	return;
	// // }
	// console.log("Reader action canceled:", cancelAction);
	const readerId = 'tmr_Fxpdsw9DI1qvaP'; // Hard-coded for demo
	try {
		// Hits your backend route that calls Stripe's readers.cancelAction(...)
		const response = await fetch('http://localhost:4242/readers/cancel-action', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ reader_id: readerId }),
		});
		const json = await response.json();
		if (json.error) {
			console.error("Error canceling action:", json.error);
			alert("Failed to cancel action. Check console for details.");
		} else {
			console.log("Reader action canceled:", json);
		}
	} catch (err) {
		console.error("Error calling /readers/cancel-action:", err);
	}
});

async function cancelReaderAction(readerId) {
	const response = await fetch('http://localhost:4242/readers/cancel-action', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ reader_id: readerId })
	});
	const { reader_state: canceledReader, error: cancelActionError } = await response.json();
	return { canceledReader, cancelActionError };
}

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
		body: JSON.stringify({ amount: amountInCents, currency: 'eur' })
	    });
	    
	    //const { clientSecret } = await response.json();
		const responseJson = await response.json();
		const clientSecret = responseJson.data?.client_secret;

		console.log("Client Secret:", clientSecret);
    
	    // Step 2: Initialize Stripe Terminal and get a connection token
	    const terminal = StripeTerminal.create({
		onFetchConnectionToken: async () => {
		    const tokenResponse = await fetch('http://localhost:4242/connection-token', {
			method: 'POST'
		    });
			const responseJson = await tokenResponse.json();
			// const { secret } = await tokenResponse.json();
		    return responseJson.data?.secret;
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

	    const collectPromise =  terminal.collectPaymentMethod(clientSecret);

		// Set a 10-second timer to auto-cancel if no tap
		const timerId = setTimeout(async () => {
			try {
				await terminal.cancelCollectPaymentMethod();
				console.log("Collect payment method canceled due to timeout");
			} catch (error) {
				console.error("Error canceling collect payment method:", error);
			}
		}, 10000);

		const collectResult = await collectPromise;
		clearTimeout(timerId);

	    if (collectResult.error) {
	        console.error("Error collecting payment method:", collectResult.error.message);
	        alert("Error collecting payment. Please try again.");
	        return;
	    }

		console.log("Payment method collected successfully:", collectResult.payment);

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
		console.error(error);
		alert("An error occurred. Please try again.");
	}
});
