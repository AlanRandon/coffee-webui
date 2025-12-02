import "htmx.org";
import "idiomorph/htmx";

const editPopup = document.getElementById(
  "edit-product-popup",
) as HTMLDialogElement;

function copyProductInfoToEditPopup(productCard: HTMLElement) {
  editPopup.querySelector<HTMLInputElement>("[data-product-id]")!.value =
    productCard.dataset.productId!;
  editPopup.querySelector<HTMLInputElement>("[data-product-name]")!.value =
    productCard.dataset.productName!;
  editPopup.querySelector<HTMLInputElement>("[data-product-price]")!.value =
    productCard.dataset.productPrice!;

  editPopup.showModal();
}

(window as any).copyProductInfoToEditPopup = copyProductInfoToEditPopup;

for (const btn of document.getElementsByTagName("button")) {
  btn.addEventListener("click", (_) => btn.blur());
}
