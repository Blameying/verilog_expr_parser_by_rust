#ifndef __ESPRESSO_BRIDGE_H
#define __ESPRESSO_BRIDGE_H

// extern void init_runtime(void);

char **run_espresso_from_data(char **data, unsigned int length,
                              unsigned int *ret_count);
char **run_espresso_from_path(char *path, unsigned int *ret_count);

#endif
