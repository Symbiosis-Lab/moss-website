// Progress Display Templates
// HTML templates for different progress states

export const progressTemplates = {
  loading: (message: string, percentage: number) => `
    <div class="progress-container">
      <style>
        .progress-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100vh;
          background: #fafafa;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          transition: all 0.3s ease;
        }

        .progress-step {
          text-align: center;
          max-width: 400px;
          padding: 32px;
          animation: fadeInUp 0.4s ease-out;
        }

        .progress-message {
          font-size: 18px;
          color: #374151;
          margin-bottom: 24px;
          font-weight: 500;
          line-height: 1.4;
        }

        .progress-bar-container {
          width: 100%;
          height: 4px;
          background: #e5e7eb;
          border-radius: 2px;
          overflow: hidden;
          margin-bottom: 16px;
        }

        .progress-bar {
          height: 100%;
          background: linear-gradient(90deg, #6366f1, #8b5cf6);
          border-radius: 2px;
          transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
          width: ${percentage}%;
        }

        .progress-percentage {
          font-size: 14px;
          color: #6b7280;
          font-weight: 500;
        }

        .spinner {
          width: 20px;
          height: 20px;
          border: 2px solid #e5e7eb;
          border-top: 2px solid #6366f1;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin: 0 auto 16px;
        }

        .completion-check {
          width: 24px;
          height: 24px;
          border-radius: 50%;
          background: #10b981;
          display: none;
          align-items: center;
          justify-content: center;
          margin: 0 auto 16px;
          color: white;
          font-size: 14px;
          animation: scaleIn 0.3s ease-out;
        }

        @keyframes fadeInUp {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }

        @keyframes scaleIn {
          from {
            opacity: 0;
            transform: scale(0.8);
          }
          to {
            opacity: 1;
            transform: scale(1);
          }
        }
      </style>

      <div class="progress-step">
        <div class="spinner"></div>
        <div class="completion-check">✓</div>
        <div class="progress-message">${message}</div>
        <div class="progress-bar-container">
          <div class="progress-bar"></div>
        </div>
        <div class="progress-percentage">${percentage}%</div>
      </div>
    </div>
  `,

  completed: (message: string, percentage: number) => `
    <div class="progress-container">
      <style>
        .progress-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100vh;
          background: #fafafa;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          transition: all 0.3s ease;
        }

        .progress-step {
          text-align: center;
          max-width: 400px;
          padding: 32px;
          animation: fadeInUp 0.4s ease-out;
        }

        .progress-message {
          font-size: 18px;
          color: #374151;
          margin-bottom: 24px;
          font-weight: 500;
          line-height: 1.4;
        }

        .progress-bar-container {
          width: 100%;
          height: 4px;
          background: #e5e7eb;
          border-radius: 2px;
          overflow: hidden;
          margin-bottom: 16px;
        }

        .progress-bar {
          height: 100%;
          background: linear-gradient(90deg, #10b981, #059669);
          border-radius: 2px;
          transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
          width: ${percentage}%;
        }

        .progress-percentage {
          font-size: 14px;
          color: #6b7280;
          font-weight: 500;
        }

        .completion-check {
          width: 24px;
          height: 24px;
          border-radius: 50%;
          background: #10b981;
          display: flex;
          align-items: center;
          justify-content: center;
          margin: 0 auto 16px;
          color: white;
          font-size: 14px;
          animation: scaleIn 0.3s ease-out;
        }

        @keyframes fadeInUp {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        @keyframes scaleIn {
          from {
            opacity: 0;
            transform: scale(0.8);
          }
          to {
            opacity: 1;
            transform: scale(1);
          }
        }
      </style>

      <div class="progress-step">
        <div class="completion-check">✓</div>
        <div class="progress-message">${message}</div>
        <div class="progress-bar-container">
          <div class="progress-bar"></div>
        </div>
        <div class="progress-percentage">${percentage}%</div>
      </div>
    </div>
  `,

  welcome: (message: string) => `
    <!DOCTYPE html>
    <html>
    <head>
      <title>moss</title>
      <style>
        body {
          font-family: -apple-system, BlinkMacSystemFont, sans-serif;
          display: flex;
          align-items: center;
          justify-content: center;
          height: 100vh;
          margin: 0;
          background: #f8f9fa;
          color: #6b7280;
        }
        .message {
          text-align: center;
          padding: 2rem;
          max-width: 500px;
        }
        .moss-icon {
          font-size: 48px;
          margin-bottom: 1rem;
        }
      </style>
    </head>
    <body>
      <div class="message">
        <div class="moss-icon">🌿</div>
        <p>${message}</p>
      </div>
    </body>
    </html>
  `,

  withSteps: (message: string, steps: any[]) => {
    const stepsHtml = steps
      .map((step: any) => {
        let stepClass = "pending";
        let icon = "⏳";

        if (step.status === "completed") {
          stepClass = "completed";
          icon = "✅";
        } else if (step.status === "active") {
          stepClass = "active";
          icon = "🔄";
        } else if (step.status === "error") {
          stepClass = "error";
          icon = "❌";
        }

        return `
        <div class="step ${stepClass}">
          <span class="step-icon">${icon}</span>
          <span>${step.name}</span>
        </div>
      `;
      })
      .join("");

    return `
      <!DOCTYPE html>
      <html>
      <head>
        <title>moss</title>
        <style>
          body {
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            background: #f8f9fa;
            color: #6b7280;
          }
          .message {
            text-align: center;
            padding: 2rem;
            max-width: 500px;
          }
          .moss-icon {
            font-size: 48px;
            margin-bottom: 1rem;
          }
          .spinner {
            border: 3px solid #e5e7eb;
            border-top: 3px solid #6366f1;
            border-radius: 50%;
            width: 24px;
            height: 24px;
            animation: spin 1s linear infinite;
            margin: 0 auto 1rem auto;
          }
          @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
          }
          .progress-steps {
            margin-top: 1.5rem;
            padding: 1rem;
            background: rgba(255, 255, 255, 0.8);
            border-radius: 8px;
            font-size: 14px;
            text-align: left;
            max-width: 300px;
          }
          .step {
            display: flex;
            align-items: center;
            margin: 0.5rem 0;
            padding: 0.25rem 0;
          }
          .step-icon {
            width: 16px;
            margin-right: 8px;
            text-align: center;
          }
          .step.completed { color: #10b981; }
          .step.active { color: #6366f1; font-weight: 500; }
          .step.pending { color: #9ca3af; }
        </style>
      </head>
      <body>
        <div class="message">
          <div class="moss-icon">🌿</div>
          <div class="spinner"></div>
          <p>${message}</p>
          <div class="progress-steps">
            <div style="font-weight: 600; margin-bottom: 0.75rem; color: #374151;">Progress:</div>
            ${stepsHtml}
          </div>
        </div>
      </body>
      </html>
    `;
  }
};