<script lang="ts">
    let {
        name,
        size,
        color,
        className
    }: PropTypes = $props();

    type PropTypes = {
        name: string;
        size: number;
        color: string;
        className: string;
    }
</script>

<svg 
  class={className}
  width={`${size}px`} 
  height="{size}px"
  fill={color}
  aria-hidden="true"
  focusable="false"
>
  <use href={`/sprite.svg#${name}`} />
</svg>