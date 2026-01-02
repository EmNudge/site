class TooltipContainer extends HTMLElement {
  constructor() {
    super();
  }
  connectedCallback() {
    import("@floating-ui/dom").then(({ computePosition, flip, shift, offset, arrow }) => {
      this.addEventListener("mouseenter", showTooltip);
      this.addEventListener("mouseleave", hideTooltip);
      this.addEventListener("focus", showTooltip);
      this.addEventListener("blur", hideTooltip);

      const regularContent = this.querySelector<HTMLElement>(".regular-content");
      const tooltipContent = this.querySelector<HTMLElement>(".tooltip");
      const arrowElement = this.querySelector<HTMLElement>(".arrow");

      async function update() {
        const { x, y, placement, middlewareData } = await computePosition(
          regularContent,
          tooltipContent,
          {
            placement: "top",
            middleware: [
              offset(6),
              flip(),
              shift({ padding: 5 }),
              arrow({ element: arrowElement }),
            ],
          },
        );

        Object.assign(tooltipContent.style, {
          left: `${x}px`,
          top: `${y}px`,
        });

        // Accessing the data
        const { x: arrowX, y: arrowY } = middlewareData.arrow;

        const staticSide = {
          top: "bottom",
          right: "left",
          bottom: "top",
          left: "right",
        }[placement.split("-")[0]];

        Object.assign(arrowElement.style, {
          left: arrowX != null ? `${arrowX}px` : "",
          top: arrowY != null ? `${arrowY}px` : "",
          right: "",
          bottom: "",
          [staticSide]: "-4px",
        });
      }

      function showTooltip() {
        tooltipContent.classList.remove("hidden");
        update();
      }

      function hideTooltip() {
        tooltipContent.classList.add("hidden");
      }
    });
  }
}

customElements.define("tooltip-container", TooltipContainer);
