<script setup lang="ts">
import InputText from 'primevue/inputtext';
import Button from 'primevue/button';
import Panel from 'primevue/panel';
import FloatLabel from 'primevue/floatlabel';
import Toast from 'primevue/toast';
import Splitter from 'primevue/splitter';
import SplitterPanel from 'primevue/splitterpanel';
import Password from 'primevue/password';
import Tree from 'primevue/tree';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import Select from 'primevue/select';
import SelectButton from 'primevue/selectbutton';
import Popover from 'primevue/popover';
import ConfirmDialog from 'primevue/confirmdialog';
import Dialog from 'primevue/dialog';
import Paginator, { PageState } from 'primevue/paginator';

import { v4 as uuidv4 } from 'uuid';
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "primevue/usetoast";
import { onMounted, onBeforeUnmount, ref } from 'vue';
import { useConfirm } from "primevue/useconfirm";


type ColumnDto = {
  index: string;
  name: string;
  type: string;
};
type KV = {
  key: string;
  value: any;
}
type ColumnSel = {
  name: string;
  type: string;
  str: KV;
  prefix_compare: KV;
  suffix_compare: KV;
  sort: KV;
};

type Node = {
  key: string;
  label: string;
  data: string;
  icon: string;
  children?: Node[];
};
type ExpandedKeys = {
  [key: string]: boolean;
};
const confirm = useConfirm();
const nodes = ref<Node[]>([]);
const selectedKey = ref(undefined);
const toast = useToast();
const islogin = ref(false);
const dbip = ref('127.0.0.1');
const username = ref('');
const password = ref('');
const dbname = ref('');
const tablename = ref('');
const sqltime = ref('');
const expandedKeys = ref<ExpandedKeys>({});
const tabledata = ref([]);
const columndata = ref<ColumnDto[]>([]);
const selectedProduct = ref();
const selected = ref<ColumnSel[]>([]);
const updateSource = ref();
const panel1Visible = ref(true);
const sqlinfo = ref('');
const total = ref(0);
const pageSize = ref(100);
const currentPage = ref(0);
const editvisible = ref(false);
const edittitle = ref('');
const addvisible = ref(false);
const addtitle = ref('');
const selvisible = ref(false);
const seltitle = ref('');
const sortOptions = ref(['升序', '降序']);
const stringOptions = ref(['等于', '不等于', '包含', '不包含', '开始以', '结束以', '是null', '不是null', '是空', '不是空']);
const compareOptions = ref(['=', '>', '<', '>=', '<=']);
const sqlvisible = ref(false);
const sqltitle = ref('');
const sqlstr = ref('');
const fileInput = ref<HTMLInputElement | null>(null);

onMounted(() => {
  nodes.value = [
    {
      key: '0',
      label: dbip.value,
      data: dbip.value,
      icon: 'pi pi-link',
      children: []
    }
  ];
});

const onNodeSelect = (node: any) => {
  if (node && node.key.length < 10) {
    if (node.key === '0') {
      dbname.value = '';
      tablename.value = '';
      selected.value = [];
      get_alldbname();
    } else {
      dbname.value = node.label;
      tablename.value = '';
      selected.value = [];
      get_alltablenamebydbname();
    }
  } else {
    if (node.label !== tablename.value) {
      selected.value = [];
    }
    tablename.value = node.label;
    let label = findParentLabelByKey(node.key, nodes.value);
    dbname.value = label ? label : "";
    setsql("SELECT * FROM " + dbname.value + "." + tablename.value);
    flushsql();
  }
};

function setsql(sql: String) {
  sqlinfo.value = sql + " LIMIT " + currentPage.value * pageSize.value + ", " + pageSize.value;
}

const onNodeUnselect = (node: any) => {
  if (node.children && node.children.length) {
    expandedKeys.value[node.key] = true;
  }
};

const contextmenuListener = ref((event: any) => {
  event.preventDefault();
});

const keydownListener = ref((event: any) => {
  // 检查按下的是否是 F5 键或者 Meta + F5
  if (event.key === 'F5' || (event.ctrlKey && event.metaKey && event.shiftKey && event.code === 'F5')) {
    event.preventDefault();
  }
});

onMounted(() => {
  //全局禁用鼠标右键
  document.addEventListener('contextmenu', contextmenuListener.value);
  // 监听键盘按下事件，禁用 F5 刷新
  document.addEventListener('keydown', keydownListener.value);
});

onBeforeUnmount(() => {
  document.removeEventListener('contextmenu', contextmenuListener.value);
  document.removeEventListener('keydown', keydownListener.value);
});

function isIPv4(inputStr: string) {
  const ipRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
  return ipRegex.test(inputStr);
}

async function connect() {
  if (!dbip.value) {
    toast.add({ severity: 'warn', summary: 'Info', detail: '请输入数据库IP地址', life: 2000 });
    return;
  }
  if (!isIPv4(dbip.value)) {
    toast.add({ severity: 'warn', summary: 'Info', detail: '请输入正确的数据库IP地址', life: 2000 });
    return;
  }
  if (!username.value) {
    toast.add({ severity: 'warn', summary: 'Info', detail: '请输入数据库用户名称', life: 2000 });
    return;
  }
  if (!password.value) {
    toast.add({ severity: 'warn', summary: 'Info', detail: '请输入数据库密码', life: 2000 });
    return;
  }
  try {
    const response = await invoke('connect_db', { dbip: dbip.value, username: username.value, password: password.value });
    if (response && response === 'true') {
      islogin.value = true;
      toast.add({ severity: 'info', summary: 'Info', detail: '连接成功', life: 2000 });
      get_alldbname();
    } else {
      toast.add({ severity: 'error', summary: 'Info', detail: '连接失败', life: 2000 });
    }
  } catch (error) {
    toast.add({ severity: 'error', summary: 'Error', detail: error, life: 4000 });
  }
}

async function get_alldbname() {
  try {
    const response = await invoke('get_alldbname');
    const treedata = JSON.parse(response as string);
    if (treedata && treedata.length > 0) {
      nodes.value[0].children = [];
      for (let i = 0; i < treedata.length; i++) {
        const newNode = {
          key: (i + 1) + "",
          label: treedata[i].name,
          data: treedata[i].name,
          icon: 'pi pi-database',
          children: []
        };
        nodes.value[0].children?.push(newNode);
      }
    }
  } catch (error) {
    toast.add({ severity: 'error', summary: 'Error', detail: error, life: 4000 });
  }
}

async function get_alltablenamebydbname() {
  try {
    const response = await invoke('get_alltablenamebydbname', { dbname: dbname.value });
    const treedata = JSON.parse(response as string);
    if (treedata && treedata.length > 0 && nodes.value.length > 0 && nodes.value[0].children) {
      const parentIndex = nodes.value[0].children.findIndex(node => node.label === dbname.value);
      if (parentIndex !== -1) {
        nodes.value[0].children[parentIndex].children = [];
        for (let i = 0; i < treedata.length; i++) {
          const newNode = {
            key: uuidv4(),
            label: treedata[i].name,
            data: treedata[i].name,
            icon: 'pi pi-table',
            children: undefined
          };
          nodes.value[0].children[parentIndex].children.push(newNode);
        }
      }
    }
  } catch (error) {
    toast.add({ severity: 'error', summary: 'Error', detail: error, life: 4000 });
  }
}

function findParentLabelByKey(key: string, nodes: Node[]): string | null {
  for (let i = 0; i < nodes.length; i++) {
    const node = nodes[i];
    if (node.children) {
      const childKeyMatch = node.children.find(child => child.key === key);
      if (childKeyMatch) {
        return node.label;
      }
      const parentLabel = findParentLabelByKey(key, node.children);
      if (parentLabel) {
        return parentLabel;
      }
    }
  }
  return null;
}

async function flushsql() {
  try {
    const startTime = performance.now();
    const response = await invoke('query_table_data', {
      dbName: dbname.value, tableName: tablename.value, sql: sqlinfo.value
    });
    const jsonres = JSON.parse(response as string);
    columndata.value = jsonres["columns"];
    tabledata.value = jsonres["data"];
    total.value = jsonres["total"];
    const endTime = performance.now();
    const executionTime = (endTime - startTime).toFixed(2);
    sqltime.value = `${executionTime} ms`;
  } catch (error) {
    toast.add({ severity: 'error', summary: 'Error', detail: error, life: 4000 });
  }
}

const opdb = ref();
const optable = ref();
const selectedDBMember = ref(null);
const selectedTableMember = ref(null);
const membersDB = ref([
  { name: '备份-数据', key: '6', icon: 'pi pi-download' },
  { name: '还原-数据', key: '7', icon: 'pi pi-download' },
  { name: '创建数据库', key: '8', icon: 'pi pi-plus' },
  { name: '删除当前库', key: '9', icon: 'pi pi-trash' }
]);
const membersTable = ref([
  { name: '重命名此表', key: '3', icon: 'pi pi-pencil' },
  { name: '清空当前表', key: '4', icon: 'pi pi-eraser' },
  { name: '删除当前表', key: '5', icon: 'pi pi-trash' },
  { name: '刷新当前表', key: '99', icon: 'pi pi-refresh' }
]);

const toggledb = (event: any) => {
  opdb.value.toggle(event);
}
const toggletable = (event: any) => {
  optable.value.toggle(event);
}

function getCurrentTimeString(): string {
  const now = new Date();
  const year = now.getFullYear();
  const month = (now.getMonth() + 1).toString().padStart(2, '0');
  const day = now.getDate().toString().padStart(2, '0');
  const hours = now.getHours().toString().padStart(2, '0');
  const minutes = now.getMinutes().toString().padStart(2, '0');
  const seconds = now.getSeconds().toString().padStart(2, '0');
  return `${year}-${month}-${day}-${hours}-${minutes}-${seconds}`;
}

async function backdb(iserr: boolean) {
  try {
    let dbn = dbname.value;
    let fln = dbname.value + "-" + getCurrentTimeString() + ".sql";
    if (tablename.value && tablename.value.length > 0) {
      dbn = dbname.value + " " + tablename.value;
      fln = dbname.value + "-" + tablename.value + "-" + getCurrentTimeString() + ".sql";
    }
    await invoke('backup_db_command', {
      username: username.value,
      password: password.value,
      dbname: dbn,
      filename: fln,
    });
    toast.add({ severity: 'info', summary: 'Info', detail: '备份操作成功', life: 2000 });
  } catch (error) {
    if (iserr) {
      toast.add({ severity: 'error', summary: 'Error', detail: '请确认设置mysqldump环境变量' + error, life: 5000 });
    }
  }
}

async function revertdb(path: string, iserr: boolean) {
  try {
    await invoke('revert_db_command', {
      username: username.value,
      password: password.value,
      dbname: dbname.value,
      filename: path,
    });
    toast.add({ severity: 'info', summary: 'Info', detail: '还原操作成功', life: 2000 });
  } catch (error) {
    if (iserr) {
      toast.add({ severity: 'error', summary: 'Error', detail: '请确认设置mysql环境变量' + error, life: 5000 });
    }
  }
}

const selectDBMember = (member: any) => {
  selectedDBMember.value = member;
  console.log(member.key + "-" + member.name);
  if (member.key === '6') {//备份当前库
    backdb(true);
  }
  if (member.key === '7') {//还原当前库
    triggerFileInput();
  }
  if (member.key === '8') {//创建数据库
    sqltitle.value = '创建数据库';
    sqlvisible.value = true;
  }
  if (member.key === '9') {//删除当前库
    confirm.require({
      message: '确认要删除该数据库吗?',
      header: '确认',
      icon: 'pi pi-exclamation-triangle',
      rejectProps: {
        label: '取消',
        severity: 'secondary',
        outlined: true
      },
      acceptProps: {
        label: '确认'
      },
      accept: () => {
        executedbtable("DROP DATABASE " + dbname.value, false);
        get_alldbname();
      },
      reject: () => {
      }
    });
  }
  opdb.value.hide();
}

function exesqlstr() {
  if (sqltitle.value === '创建数据库' && sqlstr && sqlstr.value.length > 0) {
    executedbtable("CREATE DATABASE " + sqlstr.value + " DEFAULT CHARACTER SET utf8mb4  COLLATE utf8mb4_bin ", false);
    get_alldbname();
  }
  if (sqltitle.value === '重命名此表' && sqlstr && sqlstr.value.length > 0) {
    executedbtable("RENAME TABLE " + dbname.value + "." + tablename.value + " TO " + dbname.value + "." + sqlstr.value, false);
    get_alldbname();
  }
  sqlvisible.value = false;
}

const selectTableMember = (member: any) => {
  selectedTableMember.value = member;
  console.log(member.key + "-" + member.name);
  if (member.key === '99') {//刷新当前表
    flushsql();
  }
  if (member.key === '3') {//重命名此表
    sqltitle.value = '重命名此表';
    sqlvisible.value = true;
  }
  if (member.key === '4') {//清空当前表
    executedbtable("TRUNCATE TABLE " + dbname.value + "." + tablename.value, false);
  }
  if (member.key === '5') {//删除当前表
    executedbtable("DROP TABLE " + dbname.value + "." + tablename.value, false);
  }
  optable.value.hide();
}

function paginatordeal(event: PageState) {
  currentPage.value = event.page;
  updateLimit();
  flushsql();
}

function updateLimit() {
  const limitIndex = sqlinfo.value.indexOf('LIMIT');
  if (limitIndex !== -1) {
    sqlinfo.value = sqlinfo.value.substring(0, limitIndex) + "LIMIT " + currentPage.value * pageSize.value + ", " + pageSize.value;
  }
  return '';
}

function openEdit() {
  if (selectedProduct.value) {
    updateSource.value = JSON.parse(JSON.stringify(selectedProduct.value));
    edittitle.value = "修改 " + dbname.value + "." + tablename.value + " 的数据";
    editvisible.value = true;
  } else {
    toast.add({ severity: 'info', summary: 'Info', detail: '请选择修改的数据', life: 2001 });
  }
}

function saveEdit() {
  executesql(setUpdateSql(), true);
  editvisible.value = false;
}
function openAdd() {
  addtitle.value = "添加 " + dbname.value + "." + tablename.value + " 的数据";
  selectedProduct.value = selectedProduct.value ? selectedProduct.value : {};
  addvisible.value = true;
}
function openSel() {
  seltitle.value = "查询 " + dbname.value + "." + tablename.value + " 的数据";
  //根据库表的字段类型显示当前查询页面
  if (selected.value.length <= 0) {
    columndata.value.forEach((c) => {
      let cs: ColumnSel = {
        name: c.name, type: c.type, str: {
          key: '',
          value: null
        },
        prefix_compare: {
          key: '',
          value: null
        },
        suffix_compare: {
          key: '',
          value: null
        },
        sort: {
          key: '',
          value: null
        }
      };
      selected.value.push(cs);
    });
  }
  selvisible.value = true;
}

function saveAdd() {
  console.log(selectedProduct.value);
  if (selectedProduct.value && Object.keys(selectedProduct.value).length > 0) {
    executesql(setInsertSql(), true);
  } else {
    toast.add({ severity: 'info', summary: 'Info', detail: '未添加任何数据', life: 2000 });
  }
  addvisible.value = false;
}

function ClearSql() {
  selected.value = [];
  if (selected.value.length <= 0) {
    columndata.value.forEach((c) => {
      let cs: ColumnSel = {
        name: c.name, type: c.type, str: {
          key: '',
          value: null
        },
        prefix_compare: {
          key: '',
          value: null
        },
        suffix_compare: {
          key: '',
          value: null
        },
        sort: {
          key: '',
          value: null
        }
      };
      selected.value.push(cs);
    });
  }
}

function saveSel() {
  console.log(selected.value);
  //将查询条件设置到selected
  if (selected.value && Object.keys(selected.value).length > 0) {
    setsql(setSelSql());
  } else {
    setsql("SELECT * FROM " + dbname.value + "." + tablename.value);
  }
  flushsql();
  selvisible.value = false;
}

async function executesql(sqlstr: string, iserr: boolean) {
  try {
    const response = await invoke('execute_sql_command', {
      sql: sqlstr
    });
    const jsonres = JSON.parse(response as string);
    toast.add({ severity: 'info', summary: 'Info', detail: '操作成功 受影响行数 ' + jsonres["num"], life: 2000 });
    flushsql();
  } catch (error) {
    if (iserr) {
      toast.add({ severity: 'error', summary: 'Error', detail: error, life: 5000 });
    }
  }
}
async function executedbtable(sqlstr: string, iserr: boolean) {
  try {
    await invoke('execute_dbtable_command', {
      sql: sqlstr
    });
    toast.add({ severity: 'info', summary: 'Info', detail: '操作成功', life: 2000 });
    get_alldbname();
  } catch (error) {
    if (iserr) {
      toast.add({ severity: 'error', summary: 'Error', detail: error, life: 5000 });
    }
  }
}

function setSelSql(): string {
  let str = "SELECT * FROM " + dbname.value + ".`" + tablename.value + "` WHERE 1=1 ";
  let keystr = "";//条件
  let valuestr = "";//排序
  for (const key in selected.value) {
    if (selected.value.hasOwnProperty(key)) {
      const value = selected.value[key];
      if (value['str'].key && value['str'].key !== null) {
        if (value['str'].value && value['str'].value !== null) {
          if (value['str'].key === '等于') {
            keystr = keystr + (" AND `" + value["name"] + "` = '" + value['str'].value + "'");
          }
          if (value['str'].key === '不等于') {
            keystr = keystr + (" AND `" + value["name"] + "` <> '" + value['str'].value + "'");
          }
          if (value['str'].key === '包含') {
            keystr = keystr + (" AND `" + value["name"] + "` LIKE '%" + value['str'].value + "%'");
          }
          if (value['str'].key === '不包含') {
            keystr = keystr + (" AND `" + value["name"] + "` NOT LIKE '%" + value['str'].value + "%'");
          }
          if (value['str'].key === '开始以') {
            keystr = keystr + (" AND `" + value["name"] + "` LIKE '" + value['str'].value + "%'");
          }
          if (value['str'].key === '结束以') {
            keystr = keystr + (" AND `" + value["name"] + "` LIKE '%" + value['str'].value + "'");
          }
        }
        if (value['str'].key === '是null') {
          keystr = keystr + (" AND `" + value["name"] + "` IS NULL ");
        }
        if (value['str'].key === '不是null') {
          keystr = keystr + (" AND `" + value["name"] + "` IS NOT NULL ");
        }
        if (value['str'].key === '是空') {
          keystr = keystr + (" AND `" + value["name"] + "` = '' ");
        }
        if (value['str'].key === '不是空') {
          keystr = keystr + (" AND `" + value["name"] + "` <> '' ");
        }
      }
      if (value['prefix_compare'].value && value['prefix_compare'].value !== null) {
        keystr = keystr + (" AND `" + value["name"] + "` " + value['prefix_compare'].key + " '" + value['prefix_compare'].value + "'");
      }
      if (value['suffix_compare'].value && value['suffix_compare'].value !== null) {
        keystr = keystr + (" AND `" + value["name"] + "` " + value['suffix_compare'].key + " '" + value['suffix_compare'].value + "'");
      }
      if (value['sort'].value && value['sort'].value !== null) {
        if (value['sort'].value !== '降序') {
          valuestr = valuestr + (!valuestr ? " ORDER BY `" + value["name"] + "` " : ", `" + value["name"] + "` ");
        } else {
          valuestr = valuestr + (!valuestr ? " ORDER BY `" + value["name"] + "` DESC " : ", `" + value["name"] + "` DESC");
        }
      }
    }
  }
  str = str + keystr + " " + valuestr + "";
  console.log(str);
  return str;
}
function setInsertSql(): string {
  let str = "INSERT INTO " + dbname.value + ".`" + tablename.value + "` (";
  let keystr = "";
  let valuestr = "";
  for (const key in selectedProduct.value) {
    if (selectedProduct.value.hasOwnProperty(key)) {
      const value = selectedProduct.value[key];
      keystr = keystr + (!keystr ? "`" + key + "`" : ", `" + key + "`");
      valuestr = valuestr + (!valuestr ? "'" + value + "'" : ", '" + value + "'");
    }
  }
  str = str + keystr + ") VALUES (" + valuestr + ")";
  console.log(str);
  return str;
}

function setUpdateSql(): string {
  let str = "UPDATE " + dbname.value + ".`" + tablename.value + "` SET ";
  let keystr = "";
  let valuestr = "";
  for (const key in selectedProduct.value) {
    if (selectedProduct.value.hasOwnProperty(key)) {
      const value = selectedProduct.value[key];
      if (value === null) {
        keystr = keystr + (!keystr ? "`" + key + "`=NULL" : ", `" + key + "`=NULL");
      } else {
        keystr = keystr + (!keystr ? "`" + key + "`='" + value + "'" : ", `" + key + "`='" + value + "'");
      }
      const svalue = updateSource.value[key];
      if (svalue === null) {
        valuestr = valuestr + (!valuestr ? "(ISNULL(`" + key + "`))" : " AND (ISNULL(`" + key + "`))");
      } else {
        valuestr = valuestr + (!valuestr ? "(`" + key + "`='" + svalue + "')" : " AND (`" + key + "`='" + svalue + "')");
      }
    }
  }
  str = str + keystr + " WHERE " + valuestr + " LIMIT 1";
  console.log(str);
  return str;
}

function setDelSql(): string {
  let str = "DELETE FROM " + dbname.value + ".`" + tablename.value + "` WHERE ";
  let valuestr = "";
  for (const key in selectedProduct.value) {
    if (selectedProduct.value.hasOwnProperty(key)) {
      const value = selectedProduct.value[key];
      if (value === null) {
        valuestr = valuestr + (!valuestr ? "(ISNULL(`" + key + "`))" : " AND (ISNULL(`" + key + "`))");
      } else {
        valuestr = valuestr + (!valuestr ? "(`" + key + "`='" + value + "')" : " AND (`" + key + "`='" + value + "')");
      }
    }
  }
  str = str + valuestr + " LIMIT 1";
  console.log(str);
  return str;
}

const confirmDel = () => {
  if (selectedProduct.value) {
    confirm.require({
      message: '确认要删除该数据吗?',
      header: '确认',
      icon: 'pi pi-exclamation-triangle',
      rejectProps: {
        label: '取消',
        severity: 'secondary',
        outlined: true
      },
      acceptProps: {
        label: '确认'
      },
      accept: () => {
        executesql(setDelSql(), true);
      },
      reject: () => {
      }
    });
  } else {
    toast.add({ severity: 'info', summary: 'Info', detail: '请选择删除的数据', life: 2000 });
  }
};

// 触发文件输入
const triggerFileInput = () => {
  if (fileInput.value) {
    fileInput.value.click(); // 通过点击事件触发文件选择
  }
};

// 处理文件选择变化
const handleFileChange = (event: Event) => {
  const target = event.target;
  if (target instanceof HTMLInputElement && target.files) {
    const files = target.files;
    if (files.length > 0) {
      revertdb(files[0].name, true);
    }
  }
};
</script>

<template>
  <div v-if="!islogin" class="flex justify-content-center align-items-center" style="height: 95vh !important;">
    <Panel class="w-4" header="Mysql 连接信息"
      :pt="{ content: { style: 'padding: 20px 5px 20px 5px' }, header: { style: 'font-size: 23px; justify-content: center;' } }">
      <FloatLabel>
        <InputText v-model="dbip" fluid />
        <label>数据库IP地址</label>
      </FloatLabel>
      <FloatLabel class="mt-4">
        <InputText v-model="username" fluid />
        <label>用户名</label>
      </FloatLabel>
      <FloatLabel class="mt-4">
        <Password v-model="password" fluid :feedback="false" />
        <label>密码</label>
      </FloatLabel>
      <Toast />
      <Button class="mt-4" @click="connect" type="submit" fluid>进入</Button>
    </Panel>
  </div>

  <div v-if="islogin">
    <Toast />
    <Splitter style="height: 97vh">
      <SplitterPanel :size="20" v-show="panel1Visible">
        <div class="tree-container" style="height: 100%; overflow-y: auto;">
          <Tree v-model:selectionKeys="selectedKey" v-model:expandedKeys="expandedKeys" :value="nodes"
            selectionMode="single" @nodeSelect="onNodeSelect" @nodeUnselect="onNodeUnselect" class="w-screen">
          </Tree>
        </div>
      </SplitterPanel>
      <SplitterPanel :size="80" :minSize="35">
        <DataTable :value="tabledata" scrollable scrollHeight="flex" resizableColumns columnResizeMode="expand"
          showGridlines size="small" stripedRows v-model:selection="selectedProduct" selectionMode="single"
          :pt="{ footer: { 'style': 'padding: 0px' } }">
          <template #header>
          </template>
          <template v-for="column in columndata" :key="column.name">
            <Column :field="column.name" :header="column.name" style="min-width: 2px"></Column>
          </template>
          <template #footer>
            <Paginator class="p-paginator-custom" @page="paginatordeal"
              :pt="{ root: { 'style': 'padding: 0px' }, footer: { 'style': 'padding: 0px' } }" :rows="100"
              :totalRecords="total" template="PrevPageLink CurrentPageReport NextPageLink"
              currentPageReportTemplate="{currentPage} / {totalPages} of {totalRecords}">
              <template #start>
                <Button class="ml-0 mr-3" type="button" icon="pi pi-code" text
                  @click="panel1Visible = !panel1Visible" />
                <Button v-show="dbname && tablename" class="mr-1" type="button" icon="pi pi-search" text
                  @click="openSel" />
                <Button v-show="dbname && tablename" class="mr-1" type="button" icon="pi pi-plus" text
                  @click="openAdd" />
                <Button v-show="dbname && tablename" class="mr-1" type="button" icon="pi pi-pen-to-square" text
                  @click="openEdit" />
                <Button v-show="dbname && tablename" class="mr-1" type="button" icon="pi pi-trash" text
                  @click="confirmDel" />
              </template>
              <template #end>
                <Button v-show="dbname" severity="secondary" @click="toggledb" icon="pi pi-database" :label="dbname"
                  type="button" />
                <Button v-show="dbname && tablename" severity="secondary" @click="toggletable" class="ml-2"
                  icon="pi pi-table" :label="tablename" type="button" />
              </template>
            </Paginator>
            <p style="font-size: 12px; padding: 0px; margin: 0px;">
              &nbsp;&nbsp;
              <Button style="font-size: 12px; padding: 0px; margin: 0px;" v-if="sqlinfo" @click="flushsql"
                severity="secondary">刷新
              </Button>&nbsp;&nbsp;
              {{ sqltime }} &nbsp;&nbsp;&nbsp;{{ sqlinfo }}
            </p>
          </template>
        </DataTable>
      </SplitterPanel>
    </Splitter>
  </div>
  <ConfirmDialog></ConfirmDialog>
  <input type="file" ref="fileInput" style="display: none" @change="handleFileChange" />
  <!-- 库表菜单 -->
  <Popover ref="opdb">
    <ul class=" p-0 m-0 ">
      <li v-for="member in membersDB" :key="member.name" class="flex items-center px-0 py-1"
        @click="selectDBMember(member)">
        <Button :label="member.name" severity="secondary" :icon="member.icon" fluid></Button>
      </li>
    </ul>
  </Popover>
  <Popover ref="optable">
    <ul class=" p-0 m-0 ">
      <li v-for="member in membersTable" :key="member.name" class="flex items-center px-0 py-1"
        @click="selectTableMember(member)">
        <Button :label="member.name" severity="secondary" :icon="member.icon" fluid></Button>
      </li>
    </ul>
  </Popover>
  <!-- 修改数据 -->
  <Dialog v-model:visible="editvisible" :header="edittitle" class="w-11 h-full"
    :pt="{ title: { 'style': 'padding: 0px' } }">
    <p></p>
    <template v-for="column in columndata" :key="column.name">
      <div class="flex items-center gap-4 mb-4">
        <label :for="column.name" class=" w-2 flex justify-content-end align-items-center">
          {{ column.name }}</label>
        <FloatLabel variant="on" class="flex-auto">
          <InputText :id="column.name" class="w-full" autocomplete="off" v-model="selectedProduct[column.name]" />
          <label :for="column.name">{{ column.type }}</label>
        </FloatLabel>
      </div>
    </template>

    <div class="flex justify-content-evenly gap-2">
      <Button type="button" label="取消" severity="secondary" @click="editvisible = false"></Button>
      <Button type="button" label="保存" @click="saveEdit()"></Button>
    </div>
  </Dialog>
  <!-- 添加数据 -->
  <Dialog v-model:visible="addvisible" :header="addtitle" class="w-11 h-full"
    :pt="{ title: { 'style': 'padding: 0px' } }">
    <p></p>
    <template v-for="column in columndata" :key="column.name">
      <div class="flex items-center gap-4 mb-4">
        <label :for="column.name" class=" w-2 flex justify-content-end align-items-center">
          {{ column.name }}</label>
        <FloatLabel variant="on" class="flex-auto">
          <InputText :id="column.name" class="w-full" autocomplete="off" v-model="selectedProduct[column.name]" />
          <label :for="column.name">{{ column.type }}</label>
        </FloatLabel>
      </div>
    </template>

    <div class="flex justify-content-evenly gap-2">
      <Button type="button" label="取消" severity="secondary" @click="addvisible = false"></Button>
      <Button type="button" label="保存" @click="saveAdd()"></Button>
    </div>
  </Dialog>
  <!-- 查询数据 -->
  <Dialog v-model:visible="selvisible" :header="seltitle" class="w-11 h-full"
    :pt="{ title: { 'style': 'padding: 0px' } }">
    <p></p>
    <template v-for="sd in selected" :key="sd.name">
      <div class="flex items-center gap-4 mb-4">
        <label :for="sd.name" class=" w-1 flex justify-content-end align-items-center">
          {{ sd.name }}</label>
        <div class="card flex justify-center">

          <div v-if="sd.type !== 'varchar'" class="card flex justify-center">
            <div class="card flex justify-center">
              <Select v-model="sd.str.key" :options="stringOptions" class="w-full" disabled />
              <FloatLabel variant="on" class=" w-full" disabled>
                <InputText autocomplete="off" v-model="sd.str.value" disabled />
                <label>{{ "字符串" }}</label>
              </FloatLabel>
            </div>
            <div class="card flex justify-center">
              <Select v-model="sd.prefix_compare.key" :options="compareOptions" class="w-full" />
              <FloatLabel variant="on" class=" w-full" :disabled="!sd.prefix_compare.key">
                <InputText autocomplete="off" v-model="sd.prefix_compare.value" :disabled="!sd.prefix_compare.key" />
                <label>{{ "比较" }}</label>
              </FloatLabel>
            </div>
            <div class="card flex justify-center">
              <Select v-model="sd.suffix_compare.key" :options="compareOptions" class="w-full" />
              <FloatLabel variant="on" class=" w-full" :disabled="!sd.suffix_compare.key">
                <InputText autocomplete="off" v-model="sd.suffix_compare.value" :disabled="!sd.suffix_compare.key" />
                <label>{{ "比较" }}</label>
              </FloatLabel>
            </div>
          </div>
          <div v-else class="card flex justify-center">
            <div class="card flex justify-center">
              <Select v-model="sd.str.key" :options="stringOptions" class="w-full" />
              <FloatLabel variant="on" class=" w-full" :disabled="!sd.str.key">
                <InputText autocomplete="off" v-model="sd.str.value" :disabled="!sd.str.key" />
                <label>{{ "字符串" }}</label>
              </FloatLabel>
            </div>
            <div class="card flex justify-center">
              <Select v-model="sd.prefix_compare.key" :options="compareOptions" disabled class="w-full" />
              <FloatLabel variant="on" class=" w-full" disabled>
                <InputText autocomplete="off" v-model="sd.prefix_compare.value" disabled />
                <label>{{ "比较" }}</label>
              </FloatLabel>
            </div>
            <div class="card flex justify-center">
              <Select v-model="sd.suffix_compare.key" :options="compareOptions" class="w-full" disabled />
              <FloatLabel variant="on" class=" w-full" disabled>
                <InputText autocomplete="off" v-model="sd.suffix_compare.value" disabled />
                <label>{{ "比较" }}</label>
              </FloatLabel>
            </div>
          </div>
        </div>
        <div class="card flex justify-center">
          <SelectButton v-model="sd.sort.value" :options="sortOptions" aria-labelledby="basic" />
        </div>
      </div>
    </template>
    <div class="flex justify-content-evenly gap-2">
      <Button type="button" label="清空" severity="secondary" @click="ClearSql()"></Button>
      <Button type="button" label="取消" severity="secondary" @click="selvisible = false"></Button>
      <Button type="button" label="查询" @click="saveSel()"></Button>
    </div>
  </Dialog>
  <!-- 获取一段字符 -->
  <Dialog v-model:visible="sqlvisible" modal :header="sqltitle" :style="{ width: '25rem' }">
    <div class="flex items-center gap-4 mb-4">
      <InputText id="sqlstr" class="flex-auto" autocomplete="off" v-model="sqlstr" />
    </div>
    <div class="flex justify-end gap-2">
      <Button type="button" label="取消" severity="secondary" @click="sqlstr = '', sqlvisible = false"></Button>
      <Button type="button" label="应用" @click="exesqlstr()"></Button>
    </div>
  </Dialog>
</template>
<style scoped>
.tree-container {
  overflow-y: auto;
  overflow-x: hidden;
}

.p-paginator-custom {
  border: 1px solid #e9e8e8;
  border-radius: 8px;
}
</style>
