interface MenuHandle {
  get(selector: string): { setValue(value: boolean): Promise<void> }
}

export async function pickSelectOption(menu: MenuHandle, value: string): Promise<void> {
  await menu.get(`input[type="radio"][value="${value}"]`).setValue(true)
}
