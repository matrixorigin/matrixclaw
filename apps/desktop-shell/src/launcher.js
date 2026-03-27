const LOOPBACK_SETUP_URL = "http://127.0.0.1:38495/setup";
const status = document.getElementById("status");

async function attachToLoopback() {
    try {
        const response = await fetch("http://127.0.0.1:38495/healthz");
        if (!response.ok) {
            throw new Error(`unexpected health status: ${response.status}`);
        }

        if (status) {
            status.textContent = `Opening ${LOOPBACK_SETUP_URL}`;
        }
        window.location.replace(LOOPBACK_SETUP_URL);
    } catch (error) {
        if (status) {
            status.textContent =
                "MatrixClaw loopback UI is not available yet. Start app-host, then reopen the shell.";
        }
        console.error("failed to attach to MatrixClaw loopback surface", error);
    }
}

void attachToLoopback();
