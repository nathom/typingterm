typedef struct _string {
    char *val;
    int len;
    struct _string *next;
} string;

string *get_string(string *str, int index);
string *new_string();
void insert_string(string *str, char *val, int index);
void append_string(string *str, char *val);
void print_string(string *str);
void print_strlist(string *str);
int set_string(string *str, char *val, int index);
int delete_string(string *str, int index);
