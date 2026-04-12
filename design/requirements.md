# Journal Application Requirements

## User Needs

User needs capture the core goals that drive someone to use a journal app. These remain stable even as implementation details evolve.

### 1. Capture Thoughts Quickly
- Open the app and start writing within seconds, without friction or required setup
- Save entries automatically so no writing is lost
- Add entries from any device — phone, tablet, or desktop

### 2. Reflect on Past Entries
- Browse a chronological timeline of all past entries
- Search full text across all entries to resurface specific memories or ideas
- Surface "on this day" memories automatically

### 3. Build a Consistent Habit
- See a visual writing streak or consistency tracker
- Receive optional reminders at a user-chosen time of day
- Get gentle prompts or questions to overcome writer's block

### 4. Organise and Find Entries
- Tag entries with custom labels (e.g. work, travel, gratitude)
- Filter view by tag, date range, or mood
- Pin or favourite important entries for quick access

### 5. Enrich Entries with Media
- Attach photos and images to entries
- Record mood or energy level alongside text
- Log location context automatically or manually

### 6. Trust That Data Is Private
- Lock the app with a PIN, biometric, or passphrase
- Understand clearly how data is stored and who can access it
- Export or delete all personal data at any time

---

## Product Requirements

Product requirements are concrete, testable specifications that translate user needs into system behaviour.

### 1. Entry Management
- Create, edit, and delete journal entries with rich text formatting (bold, lists, headings)
- Auto-save drafts every 5 seconds; no manual save required
- Support entries up to 50,000 characters with no performance degradation
- Allow backdating entries to any past date

### 2. Search and Retrieval
- Full-text search returning results within 300ms for up to 10,000 entries
- Filter by date range, tags, mood, and media presence
- Highlight matched search terms in results

### 3. Analytics and Insights
- Display writing streak, total word count, and entries per month
- Visualise mood trends over selectable time periods
- Surface the top 10 most-used tags on a summary dashboard

### 4. Security and Privacy
- Encrypt all entries at rest using AES-256 and in transit via TLS 1.3
- Support biometric and PIN-based app lock
- Provide a one-tap export of all data as JSON or PDF
- Comply with GDPR and CCPA data handling standards

### 5. Sync and Availability
- Sync entries across devices within 5 seconds of saving
- Support offline writing with automatic sync on reconnection
- Target 99.9% uptime for cloud sync services

### 6. Platform and Performance
- Deliver native apps for iOS and Android, and a responsive web app
- Load the home screen in under 1.5 seconds on a mid-range device
- Support accessibility standards: WCAG 2.1 AA, dynamic type, and screen readers