/* 经验贴「基本信息」列表 → 信息卡
   用 Material 的 document$ 订阅，保证 instant 导航切换页面后也生效。 */
document$.subscribe(function () {
  document.querySelectorAll("h2").forEach(function (h) {
    if (h.id === "基本信息") {
      var ul = h.nextElementSibling;
      if (ul && ul.tagName === "UL") {
        ul.classList.add("isc-info-list");
      }
    }
  });
});
