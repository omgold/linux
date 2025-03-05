// SPDX-License-Identifier: GPL-2.0

#include <linux/device.h>
#include <linux/kobject.h>
#include "../drivers/base/base.h"

int rust_helper_devm_add_action(struct device *dev,
				void (*action)(void *),
				void *data)
{
	return devm_add_action(dev, action, data);
}

struct subsys_private *rust_helper_subsys_get(struct subsys_private *sp)
{
	if (sp)
		kset_get(&sp->subsys);
	return sp;
}

void rust_helper_subsys_put(struct subsys_private *sp)
{
	if (sp)
		kset_put(&sp->subsys);
}
