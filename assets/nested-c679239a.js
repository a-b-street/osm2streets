import{S as F,i as I,s as N,e as g,y as D,n as k,a as W,b as $,z as y,A as L,G as H,B as S,h,r as A,w as B,U,E as q,V,H as R,W as Y,X as j,Y as Z,m as O,Z as X,o as T,_ as K,f as b,$ as P,t as v,q as M,k as x,L as ee,K as te,M as ne,N as le,O as se,P as re,Q as ae,R as oe,T as ie,a0 as ue}from"./MainLayers-271d044e.js";function fe(i){let e,t=i[0].size+"",n,l,a,r,u,p;return{c(){e=g("div"),n=D(t),l=D(" ways edited"),a=k(),r=g("button"),r.textContent="Download .osc",W(r,"type","button")},m(_,d){$(_,e,d),y(e,n),y(e,l),$(_,a,d),$(_,r,d),u||(p=L(r,"click",i[1]),u=!0)},p(_,[d]){d&1&&t!==(t=_[0].size+"")&&H(n,t)},i:S,o:S,d(_){_&&(h(e),h(a),h(r)),u=!1,p()}}}function ce(i,e,t){let n;A(i,B,u=>t(3,n=u));let l=new Set;function a(u){l.add(u.detail),t(0,l)}function r(){let u=`<osmChange version="0.6" generator="osm2streets">
`;u+=`<create/>
`,u+=`<modify>
`;for(let p of l)u+=n.wayToXml(p),u+=`
`;u+=`</modify>
`,u+="</osmChange>",U("lane_edits.osc",u)}return[l,r,a]}class pe extends F{constructor(e){super(),I(this,e,ce,fe,N,{handleEditedWay:2})}get handleEditedWay(){return this.$$.ctx[2]}}function z(i,e,t){const n=i.slice();return n[11]=e[t],n[12]=e,n[13]=t,n}function G(i,e){let t,n,l,a,r,u,p,_,d,f,c,s;function o(){e[5].call(l,e[12],e[13])}function m(){e[6].call(u,e[12],e[13])}function E(){return e[7](e[11])}return{key:i,first:null,c(){t=g("tr"),n=g("td"),l=g("input"),a=k(),r=g("td"),u=g("input"),p=k(),_=g("td"),d=g("button"),d.textContent="Delete",f=k(),W(l,"type","text"),W(u,"type","text"),W(d,"type","button"),this.first=t},m(C,w){$(C,t,w),y(t,n),y(n,l),j(l,e[11].key),y(t,a),y(t,r),y(r,u),j(u,e[11].value),y(t,p),y(t,_),y(_,d),y(t,f),c||(s=[L(l,"input",o),L(u,"input",m),L(d,"click",E)],c=!0)},p(C,w){e=C,w&1&&l.value!==e[11].key&&j(l,e[11].key),w&1&&u.value!==e[11].value&&j(u,e[11].value)},d(C){C&&h(t),c=!1,R(s)}}}function de(i){let e,t,n=[],l=new Map,a,r,u,p,_,d,f=q(i[0]);const c=s=>s[11].id;for(let s=0;s<f.length;s+=1){let o=z(i,f,s),m=c(o);l.set(m,n[s]=G(m,o))}return{c(){e=g("table"),t=g("tbody");for(let s=0;s<n.length;s+=1)n[s].c();a=k(),r=g("button"),r.textContent="Add new tag",u=k(),p=g("button"),p.textContent="Recalculate",W(r,"type","button"),W(p,"type","button")},m(s,o){$(s,e,o),y(e,t);for(let m=0;m<n.length;m+=1)n[m]&&n[m].m(t,null);$(s,a,o),$(s,r,o),$(s,u,o),$(s,p,o),_||(d=[L(r,"click",i[2]),L(p,"click",i[3])],_=!0)},p(s,[o]){o&3&&(f=q(s[0]),n=V(n,o,c,1,s,f,l,t,Z,G,null,z))},i:S,o:S,d(s){s&&(h(e),h(a),h(r),h(u),h(p));for(let o=0;o<n.length;o+=1)n[o].d();_=!1,R(d)}}}function _e(i,e,t){let n;A(i,B,o=>t(9,n=o));const l=Y();let{way:a}=e,r=[],u=0;for(let[o,m]of Object.entries(JSON.parse(n.getOsmTagsForWay(a))))r.push({id:u++,key:o,value:m});function p(o){t(0,r=r.filter(m=>m.id!=o))}function _(){r.push({id:u++,key:"",value:""}),t(0,r)}function d(){let o={};for(let m of r)m.key&&m.value&&(o[m.key]=m.value);n.overwriteOsmTagsForWay(a,JSON.stringify(o)),B.set(n),l("editedWay",a)}function f(o,m){o[m].key=this.value,t(0,r)}function c(o,m){o[m].value=this.value,t(0,r)}const s=o=>p(o.id);return i.$$set=o=>{"way"in o&&t(4,a=o.way)},[r,p,_,d,a,f,c,s]}class me extends F{constructor(e){super(),I(this,e,_e,de,N,{way:4})}}function ye(i){let e;return{c(){e=D("Click a road to edit")},m(t,n){$(t,e,n)},p:S,i:S,o:S,d(t){t&&h(e)}}}function ge(i){let e,t,n,l,a,r,u,p=i[0],_,d;r=new ee({props:{source:"current-way",gj:i[2],layerStyle:i[3]}});let f=J(i);return{c(){e=g("a"),t=D("Way "),n=D(i[0]),a=k(),O(r.$$.fragment),u=k(),f.c(),_=X(),W(e,"href",l="http://openstreetmap.org/way/"+i[0]),W(e,"target","_blank")},m(c,s){$(c,e,s),y(e,t),y(e,n),$(c,a,s),T(r,c,s),$(c,u,s),f.m(c,s),$(c,_,s),d=!0},p(c,s){(!d||s&1)&&H(n,c[0]),(!d||s&1&&l!==(l="http://openstreetmap.org/way/"+c[0]))&&W(e,"href",l);const o={};s&4&&(o.gj=c[2]),r.$set(o),s&1&&N(p,p=c[0])?(K(),b(f,1,1,S),P(),f=J(c),f.c(),v(f,1),f.m(_.parentNode,_)):f.p(c,s)},i(c){d||(v(r.$$.fragment,c),v(f),d=!0)},o(c){b(r.$$.fragment,c),b(f),d=!1},d(c){c&&(h(e),h(a),h(u),h(_)),M(r,c),f.d(c)}}}function J(i){let e,t;return e=new me({props:{way:i[0]}}),e.$on("editedWay",i[7]),{c(){O(e.$$.fragment)},m(n,l){T(e,n,l),t=!0},p(n,l){const a={};l&1&&(a.way=n[0]),e.$set(a)},i(n){t||(v(e.$$.fragment,n),t=!0)},o(n){b(e.$$.fragment,n),t=!1},d(n){M(e,n)}}}function $e(i){let e,t,n,l,a,r,u,p,_={};e=new pe({props:_}),i[6](e);const d=[ge,ye],f=[];function c(s,o){return s[0]?0:1}return a=c(i),r=f[a]=d[a](i),{c(){O(e.$$.fragment),t=k(),n=g("hr"),l=k(),r.c(),u=X()},m(s,o){T(e,s,o),$(s,t,o),$(s,n,o),$(s,l,o),f[a].m(s,o),$(s,u,o),p=!0},p(s,[o]){const m={};e.$set(m);let E=a;a=c(s),a===E?f[a].p(s,o):(K(),b(f[E],1,1,()=>{f[E]=null}),P(),r=f[a],r?r.p(s,o):(r=f[a]=d[a](s),r.c()),v(r,1),r.m(u.parentNode,u))},i(s){p||(v(e.$$.fragment,s),v(r),p=!0)},o(s){b(e.$$.fragment,s),b(r),p=!1},d(s){s&&(h(t),h(n),h(l),h(u)),i[6](null),M(e,s),f[a].d(s)}}}function he(i,e,t){let n,l;A(i,B,f=>t(4,n=f)),A(i,te,f=>t(5,l=f));let a,r=null,u=null,p={type:"fill",paint:{"fill-color":"red","fill-opacity":.3}};function _(f){x[f?"unshift":"push"](()=>{a=f,t(1,a)})}const d=f=>a.handleEditedWay(f);return i.$$.update=()=>{i.$$.dirty&49&&(t(0,r=null),t(2,u=null),l&&(l.properties.osm_way_ids.length!=1?window.alert("This road doesn't match up with one OSM way; you can't edit it"):(t(0,r=BigInt(l.properties.osm_way_ids[0])),t(2,u=JSON.parse(n.getGeometryForWay(r))))))},[r,a,u,p,n,l,_,d]}class ke extends F{constructor(e){super(),I(this,e,he,$e,N,{})}}function we(i){let e,t,n,l,a,r,u,p,_,d,f,c,s,o,m,E,C;return l=new se({}),p=new re({}),c=new ke({}),{c(){e=g("div"),t=g("h1"),t.textContent="osm2streets lane editor",n=k(),O(l.$$.fragment),a=k(),r=g("hr"),u=k(),O(p.$$.fragment),_=k(),d=g("hr"),f=k(),O(c.$$.fragment),s=k(),o=g("hr"),m=k(),E=g("div"),E.innerHTML="<strong>Warnings:</strong> <ul><li><strong>This tool is an early experiment</strong></li> <li>Don&#39;t use this tool without understanding OSM tagging</li> <li>Be careful around sidepaths, footways, and dual carriageways</li> <li>Don&#39;t edit a way that&#39;s partly clipped</li></ul>",W(e,"slot","left")},m(w,Q){$(w,e,Q),y(e,t),y(e,n),T(l,e,null),y(e,a),y(e,r),y(e,u),T(p,e,null),y(e,_),y(e,d),y(e,f),T(c,e,null),y(e,s),y(e,o),y(e,m),y(e,E),C=!0},p:S,i(w){C||(v(l.$$.fragment,w),v(p.$$.fragment,w),v(c.$$.fragment,w),C=!0)},o(w){b(l.$$.fragment,w),b(p.$$.fragment,w),b(c.$$.fragment,w),C=!1},d(w){w&&h(e),M(l),M(p),M(c)}}}function be(i){let e,t,n;return t=new ie({}),{c(){e=g("div"),O(t.$$.fragment),ue(e,"display","none")},m(l,a){$(l,e,a),T(t,e,null),n=!0},p:S,i(l){n||(v(t.$$.fragment,l),n=!0)},o(l){b(t.$$.fragment,l),n=!1},d(l){l&&h(e),M(t)}}}function ve(i){let e,t,n;return t=new ae({props:{$$slots:{default:[be]},$$scope:{ctx:i}}}),{c(){e=g("div"),O(t.$$.fragment),W(e,"slot","main")},m(l,a){$(l,e,a),T(t,e,null),n=!0},p(l,a){const r={};a&1&&(r.$$scope={dirty:a,ctx:l}),t.$set(r)},i(l){n||(v(t.$$.fragment,l),n=!0)},o(l){b(t.$$.fragment,l),n=!1},d(l){l&&h(e),M(t)}}}function We(i){let e,t;return e=new ne({props:{$$slots:{main:[ve],left:[we]},$$scope:{ctx:i}}}),{c(){O(e.$$.fragment)},m(n,l){T(e,n,l),t=!0},p(n,[l]){const a={};l&1&&(a.$$scope={dirty:l,ctx:n}),e.$set(a)},i(n){t||(v(e.$$.fragment,n),t=!0)},o(n){b(e.$$.fragment,n),t=!1},d(n){M(e,n)}}}function Ce(i){return le(async()=>{await oe()}),[]}class Se extends F{constructor(e){super(),I(this,e,Ce,We,N,{})}}new Se({target:document.getElementById("app")});
