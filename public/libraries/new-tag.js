/**
 * Adds "new" tags to posts that are less than 1 month old.
 * This runs client-side so tags are updated dynamically without rebuilding.
 */
export function addNewTags() {
  const posts = document.querySelectorAll("post-container[data-pub-date]");
  const now = new Date();
  const oneMonthAgo = new Date(now.getFullYear(), now.getMonth() - 1, now.getDate());

  for (const post of posts) {
    const pubDateStr = post.dataset.pubDate;
    if (!pubDateStr) continue;

    const pubDate = new Date(pubDateStr);
    if (isNaN(pubDate.getTime())) continue;

    if (pubDate > oneMonthAgo) {
      // Find the meta element to append the tag after
      const meta = post.querySelector(".meta");
      if (meta && !meta.querySelector(".new-tag")) {
        const newTag = document.createElement("span");
        newTag.className = "new-tag";
        newTag.textContent = "new";
        meta.appendChild(newTag);
      }
    }
  }
}

// Run on page load
addNewTags();
