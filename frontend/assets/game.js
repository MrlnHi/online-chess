import * as wasm from "frontend";
console.log(wasm.add(1, 2));

$(".board").delegate(".piece", "click", (e) => {
    $(".board .piece.selected").removeClass("selected");
    $(e.currentTarget).addClass("selected");
});

$(".board").on("click", (e) => {
    const piece = $(".piece.selected");
    if (piece.length == 1) {
        piece.removeClass("selected");
        const posX = Math.floor(e.offsetX / e.currentTarget.clientWidth * 8);
        const posY = Math.floor((1 - e.offsetY / e.currentTarget.clientHeight) * 8);
        piece.css("transform", `translate(${posX * 100}%, ${(7 - posY) * 100}%)`);
    }
});

$(document).on("click", (e) => {
    if ($(e.target).hasClass("piece")) {
        return;
    }
    $(".board .piece.selected").removeClass("selected");
});