/**
 * Formats a date string consistently across the site
 * @param dateString - Any valid date string (e.g., "Dec 31, 2025", "2025-12-31", etc.)
 * @returns Formatted date string in "Month Day, Year" format (e.g., "December 31, 2025")
 */
export function formatDate(dateString: string): string {
  const date = new Date(dateString);

  // Check if date is valid
  if (isNaN(date.getTime())) {
    console.warn(`Invalid date string: ${dateString}`);
    return dateString; // Return original string if invalid
  }

  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}
