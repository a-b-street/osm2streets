import{S as F,i as J,s as R,p as y,y as j,M as _,Z as E,a as w,B as m,D as N,af as X,E as I,d as k,j as z,R as D,ag as ee,a0 as H,ah as te,a2 as Z,h as ne,z as B,ai as ae,K as C,e as Q,N as M,g as Y,b,c as x,t as v,P as S,x as le,a3 as re,o as se,T as oe,a4 as ie,a5 as ue,ad as fe,a6 as de,a7 as ce,a8 as pe,a9 as $e,aa as me,ab as ge,ac as _e,I as ye,ae as he,aj as we,U as ke,V as be,W as ve}from"./RenderLanePolygons-6addba79.js";function We(u){let e,t=u[0].size+"",n,a,l,r,s,d;return{c(){e=y("div"),n=j(t),a=j(" ways edited"),l=_(),r=y("button"),r.textContent="Download .osc",E(r,"type","button")},m(c,f){w(c,e,f),m(e,n),m(e,a),w(c,l,f),w(c,r,f),s||(d=N(r,"click",u[1]),s=!0)},p(c,[f]){f&1&&t!==(t=c[0].size+"")&&X(n,t)},i:I,o:I,d(c){c&&(k(e),k(l),k(r)),s=!1,d()}}}function Ce(u,e,t){let n;z(u,D,s=>t(3,n=s));let a=new Set;function l(s){a.add(s.detail),t(0,a)}function r(){let s=`<osmChange version="0.6" generator="osm2streets">
`;s+=`<create/>
`,s+=`<modify>
`;for(let d of a)s+=n.wayToXml(d),s+=`
`;s+=`</modify>
`,s+="</osmChange>",ee("lane_edits.osc",s)}return[a,r,l]}class Me extends F{constructor(e){super(),J(this,e,Ce,We,R,{handleEditedWay:2})}get handleEditedWay(){return this.$$.ctx[2]}}function V(u,e,t){const n=u.slice();return n[11]=e[t],n[12]=e,n[13]=t,n}function K(u,e){let t,n,a,l,r,s,d,c,f,p,h,i;function o(){e[5].call(a,e[12],e[13])}function g(){e[6].call(s,e[12],e[13])}function $(){return e[7](e[11])}return{key:u,first:null,c(){t=y("tr"),n=y("td"),a=y("input"),l=_(),r=y("td"),s=y("input"),d=_(),c=y("td"),f=y("button"),f.textContent="Delete",p=_(),E(a,"type","text"),E(s,"type","text"),E(f,"type","button"),this.first=t},m(W,T){w(W,t,T),m(t,n),m(n,a),B(a,e[11].key),m(t,l),m(t,r),m(r,s),B(s,e[11].value),m(t,d),m(t,c),m(c,f),m(t,p),h||(i=[N(a,"input",o),N(s,"input",g),N(f,"click",$)],h=!0)},p(W,T){e=W,T&1&&a.value!==e[11].key&&B(a,e[11].key),T&1&&s.value!==e[11].value&&B(s,e[11].value)},d(W){W&&k(t),h=!1,Z(i)}}}function Se(u){let e,t,n=[],a=new Map,l,r,s,d,c,f,p=H(u[0]);const h=i=>i[11].id;for(let i=0;i<p.length;i+=1){let o=V(u,p,i),g=h(o);a.set(g,n[i]=K(g,o))}return{c(){e=y("table"),t=y("tbody");for(let i=0;i<n.length;i+=1)n[i].c();l=_(),r=y("button"),r.textContent="Add new tag",s=_(),d=y("button"),d.textContent="Recalculate",E(r,"type","button"),E(d,"type","button")},m(i,o){w(i,e,o),m(e,t);for(let g=0;g<n.length;g+=1)n[g]&&n[g].m(t,null);w(i,l,o),w(i,r,o),w(i,s,o),w(i,d,o),c||(f=[N(r,"click",u[2]),N(d,"click",u[3])],c=!0)},p(i,[o]){o&3&&(p=H(i[0]),n=te(n,o,h,1,i,p,a,t,ae,K,null,V))},i:I,o:I,d(i){i&&(k(e),k(l),k(r),k(s),k(d));for(let o=0;o<n.length;o+=1)n[o].d();c=!1,Z(f)}}}function Oe(u,e,t){let n;z(u,D,o=>t(9,n=o));const a=ne();let{way:l}=e,r=[],s=0;for(let[o,g]of Object.entries(JSON.parse(n.getOsmTagsForWay(l))))r.push({id:s++,key:o,value:g});function d(o){t(0,r=r.filter(g=>g.id!=o))}function c(){r.push({id:s++,key:"",value:""}),t(0,r)}function f(){let o={};for(let g of r)g.key&&g.value&&(o[g.key]=g.value);n.overwriteOsmTagsForWay(l,JSON.stringify(o)),D.set(n),a("editedWay",l)}function p(o,g){o[g].key=this.value,t(0,r)}function h(o,g){o[g].value=this.value,t(0,r)}const i=o=>d(o.id);return u.$$set=o=>{"way"in o&&t(4,l=o.way)},[r,d,c,f,l,p,h,i]}class Te extends F{constructor(e){super(),J(this,e,Oe,Se,R,{way:4})}}function Ee(u){let e;return{c(){e=j("Click a road to edit")},m(t,n){w(t,e,n)},p:I,i:I,o:I,d(t){t&&k(e)}}}function Ie(u){let e,t,n,a,l,r=u[0],s,d,c=U(u);return{c(){e=y("a"),t=j("Way "),n=j(u[0]),l=_(),c.c(),s=Q(),E(e,"href",a="http://openstreetmap.org/way/"+u[0]),E(e,"target","_blank")},m(f,p){w(f,e,p),m(e,t),m(e,n),w(f,l,p),c.m(f,p),w(f,s,p),d=!0},p(f,p){(!d||p&1)&&X(n,f[0]),(!d||p&1&&a!==(a="http://openstreetmap.org/way/"+f[0]))&&E(e,"href",a),p&1&&R(r,r=f[0])?(Y(),b(c,1,1,I),x(),c=U(f),c.c(),v(c,1),c.m(s.parentNode,s)):c.p(f,p)},i(f){d||(v(c),d=!0)},o(f){b(c),d=!1},d(f){f&&(k(e),k(l),k(s)),c.d(f)}}}function U(u){let e,t;return e=new Te({props:{way:u[0]}}),e.$on("editedWay",u[3]),{c(){C(e.$$.fragment)},m(n,a){M(e,n,a),t=!0},p(n,a){const l={};a&1&&(l.way=n[0]),e.$set(l)},i(n){t||(v(e.$$.fragment,n),t=!0)},o(n){b(e.$$.fragment,n),t=!1},d(n){S(e,n)}}}function Le(u){let e,t,n,a,l,r,s,d,c={};e=new Me({props:c}),u[2](e);const f=[Ie,Ee],p=[];function h(i,o){return i[0]?0:1}return l=h(u),r=p[l]=f[l](u),{c(){C(e.$$.fragment),t=_(),n=y("hr"),a=_(),r.c(),s=Q()},m(i,o){M(e,i,o),w(i,t,o),w(i,n,o),w(i,a,o),p[l].m(i,o),w(i,s,o),d=!0},p(i,[o]){const g={};e.$set(g);let $=l;l=h(i),l===$?p[l].p(i,o):(Y(),b(p[$],1,1,()=>{p[$]=null}),x(),r=p[l],r?r.p(i,o):(r=p[l]=f[l](i),r.c()),v(r,1),r.m(s.parentNode,s))},i(i){d||(v(e.$$.fragment,i),v(r),d=!0)},o(i){b(e.$$.fragment,i),b(r),d=!1},d(i){i&&(k(t),k(n),k(a),k(s)),u[2](null),S(e,i),p[l].d(i)}}}function Ne(u,e,t){let{way:n}=e,a;function l(s){le[s?"unshift":"push"](()=>{a=s,t(1,a)})}const r=s=>a.handleEditedWay(s);return u.$$set=s=>{"way"in s&&t(0,n=s.way)},[n,a,l,r]}class je extends F{constructor(e){super(),J(this,e,Ne,Le,R,{way:0})}}function De(u){let e,t,n,a,l,r,s,d,c,f,p,h,i,o,g,$,W,T,P,L,G;return a=new ie({}),f=new ue({}),o=new je({props:{way:u[0]}}),L=new fe({}),{c(){e=y("div"),t=y("h1"),t.textContent="osm2streets lane editor",n=_(),C(a.$$.fragment),l=_(),r=y("p"),r.innerHTML=`Improve OSM lane tagging with
      <a href="https://github.com/a-b-street/osm2streets" target="_blank">osm2streets</a>
      and <a href="https://gitlab.com/LeLuxNet/Muv" target="_blank">Muv</a>.`,s=_(),d=y("hr"),c=_(),C(f.$$.fragment),p=_(),h=y("hr"),i=_(),C(o.$$.fragment),g=_(),$=y("hr"),W=_(),T=y("div"),T.innerHTML="<strong>Warnings:</strong> <ul><li><strong>This tool is an early experiment</strong></li> <li>Don&#39;t use this tool without understanding OSM tagging</li> <li>Be careful around sidepaths, footways, and dual carriageways</li> <li>Don&#39;t edit a way that&#39;s partly clipped</li></ul>",P=_(),C(L.$$.fragment),E(e,"slot","left")},m(O,A){w(O,e,A),m(e,t),m(e,n),M(a,e,null),m(e,l),m(e,r),m(e,s),m(e,d),m(e,c),M(f,e,null),m(e,p),m(e,h),m(e,i),M(o,e,null),m(e,g),m(e,$),m(e,W),m(e,T),m(e,P),M(L,e,null),G=!0},p(O,A){const q={};A&1&&(q.way=O[0]),o.$set(q)},i(O){G||(v(a.$$.fragment,O),v(f.$$.fragment,O),v(o.$$.fragment,O),v(L.$$.fragment,O),G=!0)},o(O){b(a.$$.fragment,O),b(f.$$.fragment,O),b(o.$$.fragment,O),b(L.$$.fragment,O),G=!1},d(O){O&&k(e),S(a),S(f),S(o),S(L)}}}function Re(u){let e,t;const n=[ke("current-way"),{paint:{"fill-color":"red","fill-opacity":.3}}];let a={};for(let l=0;l<n.length;l+=1)a=be(a,n[l]);return e=new ve({props:a}),{c(){C(e.$$.fragment)},m(l,r){M(e,l,r),t=!0},p(l,r){const s={};e.$set(s)},i(l){t||(v(e.$$.fragment,l),t=!0)},o(l){b(e.$$.fragment,l),t=!1},d(l){S(e,l)}}}function Ge(u){let e,t,n,a,l,r,s,d,c,f,p,h,i,o,g;return t=new pe({}),a=new $e({}),r=new me({}),d=new ge({props:{hoverCursor:"pointer"}}),d.$on("click",u[2]),f=new _e({}),h=new ye({props:{data:u[1],$$slots:{default:[Re]},$$scope:{ctx:u}}}),o=new he({}),{c(){e=y("div"),C(t.$$.fragment),n=_(),C(a.$$.fragment),l=_(),C(r.$$.fragment),s=_(),C(d.$$.fragment),c=_(),C(f.$$.fragment),p=_(),C(h.$$.fragment),i=_(),C(o.$$.fragment),we(e,"display","none")},m($,W){w($,e,W),M(t,e,null),m(e,n),M(a,e,null),m(e,l),M(r,e,null),m(e,s),M(d,e,null),m(e,c),M(f,e,null),m(e,p),M(h,e,null),w($,i,W),M(o,$,W),g=!0},p($,W){const T={};W&2&&(T.data=$[1]),W&16&&(T.$$scope={dirty:W,ctx:$}),h.$set(T)},i($){g||(v(t.$$.fragment,$),v(a.$$.fragment,$),v(r.$$.fragment,$),v(d.$$.fragment,$),v(f.$$.fragment,$),v(h.$$.fragment,$),v(o.$$.fragment,$),g=!0)},o($){b(t.$$.fragment,$),b(a.$$.fragment,$),b(r.$$.fragment,$),b(d.$$.fragment,$),b(f.$$.fragment,$),b(h.$$.fragment,$),b(o.$$.fragment,$),g=!1},d($){$&&(k(e),k(i)),S(t),S(a),S(r),S(d),S(f),S(h),S(o,$)}}}function Be(u){let e,t,n;return t=new de({props:{$$slots:{default:[Ge]},$$scope:{ctx:u}}}),{c(){e=y("div"),C(t.$$.fragment),E(e,"slot","main")},m(a,l){w(a,e,l),M(t,e,null),n=!0},p(a,l){const r={};l&18&&(r.$$scope={dirty:l,ctx:a}),t.$set(r)},i(a){n||(v(t.$$.fragment,a),n=!0)},o(a){b(t.$$.fragment,a),n=!1},d(a){a&&k(e),S(t)}}}function Fe(u){let e,t;return e=new re({props:{$$slots:{main:[Be],left:[De]},$$scope:{ctx:u}}}),{c(){C(e.$$.fragment)},m(n,a){M(e,n,a),t=!0},p(n,[a]){const l={};a&19&&(l.$$scope={dirty:a,ctx:n}),e.$set(l)},i(n){t||(v(e.$$.fragment,n),t=!0)},o(n){b(e.$$.fragment,n),t=!1},d(n){S(e,n)}}}function Je(u,e,t){let n,a;z(u,D,s=>t(3,a=s)),se(async()=>{await ce()});let l=null;D.subscribe(s=>{t(0,l=null)});function r(s){let d=JSON.parse(s.detail.features[0].properties.osm_way_ids);d.length!=1?window.alert("This road doesn't match up with one OSM way; you can't edit it"):t(0,l=BigInt(d[0]))}return u.$$.update=()=>{u.$$.dirty&9&&t(1,n=l?JSON.parse(a.getGeometryForWay(l)):oe())},[l,n,r,a]}class Ae extends F{constructor(e){super(),J(this,e,Je,Fe,R,{})}}new Ae({target:document.getElementById("app")});
