package writes

import org.scalatest.{FlatSpec, Matchers}
import play.api.libs.json.JsValue
import util._

class CreateMutationSpec extends FlatSpec with Matchers with ApiSpecBase {
  val schema =
    """
    |model ScalarModel {
    |   id          String @id @default(cuid())
    |   optString   String?
    |   optInt      Int?
    |   optFloat    Float?
    |   optBoolean  Boolean?
    |   optEnum     MyEnum?
    |   optDateTime DateTime?
    |   optUnique   String? @unique
    |   createdAt   DateTime @default(now())
    |}
    |
    |enum MyEnum {
    |   A
    |   B
    |}""".stripMargin

  val project = ProjectDsl.fromString { schema }

  override protected def beforeAll(): Unit = {
    super.beforeAll()
    database.setup(project)
  }

  override def beforeEach(): Unit = database.truncateProjectTables(project)

  "A Create Mutation" should "create and return item" in {
    val res = server.query(
      s"""mutation {
         |  createScalarModel(data: {
         |    optString: "lala${TroubleCharacters.value}", optInt: 1337, optFloat: 1.234, optBoolean: true, optEnum: A, optDateTime: "2016-07-31T23:59:01.000Z"
         |  }){id, optString, optInt, optFloat, optBoolean, optEnum, optDateTime }
         |}""".stripMargin,
      project = project
    )
    val id = res.pathAsString("data.createScalarModel.id")

    res should be(
      s"""{"data":{"createScalarModel":{"id":"$id","optInt":1337,"optBoolean":true,"optDateTime":"2016-07-31T23:59:01.000Z","optString":"lala${TroubleCharacters.value}","optEnum":"A","optFloat":1.234}}}""".parseJson)

    val queryRes = server.query("""{ scalarModels{optString, optInt, optFloat, optBoolean, optEnum, optDateTime }}""", project = project)

    queryRes should be(
      s"""{"data":{"scalarModels":[{"optInt":1337,"optBoolean":true,"optDateTime":"2016-07-31T23:59:01.000Z","optString":"lala${TroubleCharacters.value}","optEnum":"A","optFloat":1.234}]}}""".parseJson)
  }

  "A Create Mutation" should "create and return item with empty string" in {
    val res = server.query(
      """mutation {
        |  createScalarModel(data: {
        |    optString: ""
        |  }){optString, optInt, optFloat, optBoolean, optEnum }}""".stripMargin,
      project = project
    )

    res should be("""{"data":{"createScalarModel":{"optInt":null,"optBoolean":null,"optString":"","optEnum":null,"optFloat":null}}}""".parseJson)
  }

  "A Create Mutation" should "create and return item with explicit null attributes" in {
    val res = server.query(
      """mutation {
        |  createScalarModel(data: {
        |    optString: null, optInt: null, optBoolean: null, optEnum: null, optFloat: null
        |  }){optString, optInt, optFloat, optBoolean, optEnum}}""".stripMargin,
      project
    )

    res should be("""{"data":{"createScalarModel":{"optInt":null,"optBoolean":null,"optString":null,"optEnum":null,"optFloat":null}}}""".parseJson)
  }

  "A Create Mutation" should "create and return item with explicit null attributes when other mutation has explicit non-null values" in {
    val res = server.query(
      """mutation {
        | a: createScalarModel(data: {optString: "lala", optInt: 123, optBoolean: true, optEnum: A, optFloat: 1.23}){optString, optInt, optFloat, optBoolean, optEnum }
        | b: createScalarModel(data: {optString: null, optInt: null, optBoolean: null, optEnum: null, optFloat: null}){optString, optInt, optFloat, optBoolean, optEnum }
        |}""".stripMargin,
      project = project
    )

    res.pathAs[JsValue]("data.a") should be("""{"optInt":123,"optBoolean":true,"optString":"lala","optEnum":"A","optFloat":1.23}""".parseJson)
    res.pathAs[JsValue]("data.b") should be("""{"optInt":null,"optBoolean":null,"optString":null,"optEnum":null,"optFloat":null}""".parseJson)
  }

  "A Create Mutation" should "create and return item with implicit null attributes and createdAt should be set" in {
    val res = server.query("""mutation {createScalarModel(data:{}){ optString, optInt, optFloat, optBoolean, optEnum }}""", project)

    // if the query succeeds createdAt did work. If would not have been set we would get a NullConstraintViolation.
    res should be("""{"data":{"createScalarModel":{"optInt":null,"optBoolean":null,"optString":null,"optEnum":null,"optFloat":null}}}""".parseJson)
  }

  "A Create Mutation" should "fail when a DateTime is invalid" in {
    server.queryThatMustFail(
      s"""mutation { createScalarModel(data:
         |  { optString: "test", optInt: 1337, optFloat: 1.234, optBoolean: true, optEnum: A, optDateTime: "2016-0B-31T23:59:01.000Z" }
         |  ){optString, optInt, optFloat, optBoolean, optEnum, optDateTime}}""".stripMargin,
      project = project,
      0,
      errorContains = "Invalid DateTime: input contains invalid characters DateTime must adhere to format"
    )
  }

  "A Create Mutation" should "fail when an Int is invalid" in {
    server.queryThatMustFail(
      s"""mutation {createScalarModel(data: {optString: "test", optInt: B, optFloat: 1.234, optBoolean: true, optEnum: A, optDateTime: "2016-07-31T23:59:01.000Z" }){optString, optInt, optFloat, optBoolean, optEnum, optDateTime }}""",
      project = project,
      0,
      errorContains = """Value types mismatch. Have: Enum("B"), want: Scalar(Int)"""
    )
  }

  "A Create Mutation" should "gracefully fail when a unique violation occurs" in {
    val mutation = s"""mutation {createScalarModel(data: {optUnique: "test"}){optUnique}}"""
    server.query(mutation, project)
    server.queryThatMustFail(mutation, project, errorCode = 3010)
  }
}
