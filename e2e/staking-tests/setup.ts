import { destroyClient } from "shared";

after(() => {
  destroyClient();
});
