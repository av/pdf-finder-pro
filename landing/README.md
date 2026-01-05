# Landing Page

This folder contains the landing page for PDF Finder Pro.

## Purpose

Marketing page to present the application to potential users. Focused on clearly explaining what the app does and why someone would use it.

## Design Principles

The landing page follows these guidelines:

1. **Clear Communication**: Direct language that explains benefits, not just features
2. **Simple Layout**: Clean visual hierarchy that guides the eye
3. **No Gimmicks**: Straightforward copy without trying to be clever or trendy
4. **Conversion Focus**: Clear path from visitor to download

## Files

- `index.html` - Landing page structure and content
- `landing.css` - Styles following clean, professional design principles

## Viewing the Landing Page

Open `index.html` in a web browser to view the landing page locally.

For development, you can serve it with any static file server:

```bash
# Using Python
python -m http.server 8000

# Using Node.js http-server
npx http-server
```

Then navigate to http://localhost:8000/landing/ in your browser.

## Content Structure

1. **Hero** - Main value proposition and call to action
2. **Problem/Solution** - Direct comparison showing what problem this solves
3. **Features** - What the app does in clear terms
4. **How It Works** - Simple 3-step explanation
5. **Technical Details** - Requirements and technology for transparency
6. **Download** - Clear download section with GitHub links
7. **Footer** - Navigation and legal information

## Design Notes

- Colors match the main application (orange primary, blue secondary)
- Typography uses system fonts for fast loading and native feel
- Layout is responsive for mobile, tablet, and desktop
- No JavaScript required - pure HTML/CSS
- Accessible with keyboard navigation and screen readers
