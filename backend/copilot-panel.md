If your goal is to embed an AI Agent into any web application as a floating button in the **top-right corner**, which expands into a full interactive panel when clicked, here's a complete integration template you can use immediately.

### Features

* Floating AI button (top-right)
* Expandable/collapsible panel
* Chat interface
* Agent status indicator
* Tool/action buttons
* API integration placeholder
* Mobile responsive

---

# Component Structure

```text
src/
 ├── components/
 │    ├── AIAgentPanel.jsx
 │    ├── AIAgentPanel.css
 │    └── agentApi.js
 │
 └── App.jsx
```

---

# AIAgentPanel.jsx

```jsx
import React, { useState } from "react";
import "./AIAgentPanel.css";

export default function AIAgentPanel() {
  const [open, setOpen] = useState(false);
  const [message, setMessage] = useState("");
  const [chat, setChat] = useState([
    {
      role: "agent",
      text: "Hello. How can I help you today?"
    }
  ]);

  const sendMessage = async () => {
    if (!message.trim()) return;

    const userMessage = {
      role: "user",
      text: message
    };

    setChat(prev => [...prev, userMessage]);

    const prompt = message;
    setMessage("");

    // Replace with your AI endpoint
    const response =
      "This is where your AI Agent response will appear.";

    setChat(prev => [
      ...prev,
      {
        role: "agent",
        text: response
      }
    ]);
  };

  return (
    <>
      <button
        className="agent-toggle"
        onClick={() => setOpen(!open)}
      >
        AI
      </button>

      <div className={`agent-panel ${open ? "open" : ""}`}>
        <div className="agent-header">
          <div>
            <h3>AI Agent</h3>
            <span className="status">● Online</span>
          </div>

          <button
            className="close-btn"
            onClick={() => setOpen(false)}
          >
            ✕
          </button>
        </div>

        <div className="agent-actions">
          <button>Analyze</button>
          <button>Search</button>
          <button>Generate</button>
          <button>Report</button>
        </div>

        <div className="chat-area">
          {chat.map((msg, index) => (
            <div
              key={index}
              className={`msg ${msg.role}`}
            >
              {msg.text}
            </div>
          ))}
        </div>

        <div className="input-area">
          <input
            value={message}
            onChange={(e) =>
              setMessage(e.target.value)
            }
            placeholder="Ask your AI Agent..."
          />

          <button onClick={sendMessage}>
            Send
          </button>
        </div>
      </div>
    </>
  );
}
```

---

# AIAgentPanel.css

```css
.agent-toggle {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 9999;

  width: 60px;
  height: 60px;
  border-radius: 50%;

  border: none;
  cursor: pointer;

  background: #1a73e8;
  color: white;
  font-size: 18px;
}

.agent-panel {
  position: fixed;
  top: 20px;
  right: 20px;

  width: 0;
  height: 700px;

  overflow: hidden;

  background: white;
  border-radius: 16px;

  box-shadow: 0 10px 30px rgba(0,0,0,.2);

  transition: all .35s ease;
  z-index: 9998;
}

.agent-panel.open {
  width: 420px;
}

.agent-header {
  display: flex;
  justify-content: space-between;
  align-items: center;

  padding: 15px;
  border-bottom: 1px solid #eee;
}

.status {
  color: green;
  font-size: 12px;
}

.close-btn {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 18px;
}

.agent-actions {
  display: flex;
  gap: 10px;
  padding: 10px;
}

.agent-actions button {
  padding: 8px 12px;
}

.chat-area {
  height: 500px;
  overflow-y: auto;
  padding: 15px;
}

.msg {
  margin-bottom: 12px;
  padding: 10px;
  border-radius: 10px;
}

.msg.user {
  background: #dceeff;
}

.msg.agent {
  background: #f2f2f2;
}

.input-area {
  display: flex;
  padding: 15px;
  gap: 10px;
}

.input-area input {
  flex: 1;
  padding: 10px;
}
```

---

# OpenAI / AI Agent Integration Layer

```javascript
export async function askAgent(prompt) {

  const response = await fetch(
    "/api/agent",
    {
      method: "POST",
      headers: {
        "Content-Type":
        "application/json"
      },
      body: JSON.stringify({ prompt })
    }
  );

  return response.json();
}
```

---

# Enterprise Agent Version

You can extend this panel with:

### Left Navigation

```text
Dashboard
Projects
Documents
CRM
Reports
Mining Data
Financial Model
Investors
```

### Agent Tools

```text
✓ Web Search
✓ Document Reader
✓ PDF Analysis
✓ Excel Analysis
✓ CRM Actions
✓ Email Sending
✓ Workflow Automation
✓ Database Queries
✓ Financial Modeling
✓ Investor Deck Generator
```

### Modes

```text
Chat Mode
Research Mode
Analyst Mode
Executive Mode
Automation Mode
```

---

# SaaS-Style Layout

```text
┌─────────────────────────────────────┐
│ Your Application                     │
│                                      │
│                             [ AI ]   │
│                                      │
└─────────────────────────────────────┘

Click AI

┌─────────────────────────────────────┬──────────────┐
│ Your Application                     │ AI Agent     │
│                                      │──────────────│
│                                      │ Chat         │
│                                      │ Files        │
│                                      │ Tools        │
│                                      │ Actions      │
│                                      │ Workflows    │
└─────────────────────────────────────┴──────────────┘
```

This pattern is what many modern AI copilots use: a floating launcher that expands into a persistent side panel without taking the user away from the application.
